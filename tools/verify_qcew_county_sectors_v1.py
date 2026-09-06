#!/usr/bin/env python3
"""Read-only verification of the Michigan private county-sector reference.

Ordinary verification uses checked artifact and manifest bytes only. The optional
--source-dir explicitly authorizes source verification and a temporary rebuild;
it never rewrites the checked artifact or another registry entry.
"""

from __future__ import annotations

import argparse
import csv
import gzip
import hashlib
import io
import json
from dataclasses import asdict
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any, Final

import make_qcew_county_sector_artifacts as builder
import yaml
from make_qcew_county_economics_artifacts import (
    ASCII_DIGITS,
    COUNTY_FILE_RE,
    MICHIGAN_COUNTY_GEOIDS,
    QcewBuildError,
    load_source_manifest,
)
from yaml.constructor import ConstructorError
from yaml.nodes import MappingNode

CONTRACT_PATH: Final = "contracts/qcew_county_sectors_v1.yaml"
SOURCE_MANIFEST_PATH: Final = "tools/qcew_county_economics_v1_source_manifest.json"
MAX_ARTIFACT_BYTES: Final = 1_048_576
EXPECTED_CENSUS: Final = {
    "rows": 1603,
    "counties": 83,
    "disclosed_rows": 1187,
    "suppressed_rows": 416,
    "classified_rows": 1522,
    "unclassified_rows": 81,
    "absent_cells": 57,
}
EXPECTED_SEMANTICS: Final = {
    "numeric_conversion": "exact-nonnegative-i64-no-rounding",
    "annual_avg_estabs_count": "annual-average-establishments",
    "annual_avg_emplvl": "annual-average-jobs-not-distinct-people",
    "total_annual_wages": "calendar-year-total-USD",
    "annual_avg_wkly_wage": "annual-average-USD-per-employee-per-week-not-median",
    "suppression": "N-retains-establishments-other-three-metrics-CSV-empty-JSON-null",
    "disclosed_zero": "exact-observed-zero",
    "absent_cell": "no-row-not-zero",
    "unclassified": "code-99-preserved-with-unclassified-disposition",
    "combined_codes": "preserved-no-prefix-truncation-or-detail-rollup",
    "provenance": "every-row-binds-exact-county-source-filename-and-sha256",
    "time_policy": "fixed-observed-2024-baseline",
}


class _UniqueLoader(yaml.SafeLoader):
    """Reject duplicate YAML keys before evaluating contract identity."""


def _unique_mapping(loader: _UniqueLoader, node: MappingNode, deep: bool = False) -> dict[Any, Any]:
    result: dict[Any, Any] = {}
    for key_node, value_node in node.value:
        key = loader.construct_object(key_node, deep=deep)
        if key in result:
            raise ConstructorError(None, None, f"duplicate key {key!r}", key_node.start_mark)
        result[key] = loader.construct_object(value_node, deep=deep)
    return result


_UniqueLoader.add_constructor(yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, _unique_mapping)


def _bounded_bytes(path: Path, maximum: int) -> bytes:
    try:
        with path.open("rb") as handle:
            raw = handle.read(maximum + 1)
    except OSError as error:
        raise QcewBuildError("file_read", str(path)) from error
    if len(raw) > maximum:
        raise QcewBuildError("file_size", str(path))
    return raw


def _load_mapping(path: Path, maximum: int) -> dict[str, Any]:
    raw = _bounded_bytes(path, maximum)
    try:
        document = yaml.load(raw, Loader=_UniqueLoader)  # noqa: S506 -- SafeLoader subclass
    except (yaml.YAMLError, UnicodeDecodeError) as error:
        raise QcewBuildError("invalid_yaml", str(path)) from error
    if not isinstance(document, dict) or not all(isinstance(key, str) for key in document):
        raise QcewBuildError("mapping_shape", str(path))
    return document


def load_contract(path: Path) -> dict[str, Any]:
    """Read a bounded contract without consulting any acquisition directory."""
    return _load_mapping(path, 65_536)


def _equal(actual: object, expected: object, code: str) -> None:
    try:
        matches = json.dumps(actual, sort_keys=True, allow_nan=False) == json.dumps(
            expected, sort_keys=True, allow_nan=False
        )
    except (TypeError, ValueError) as error:
        raise QcewBuildError(code, "noncanonical contract value") from error
    if not matches:
        raise QcewBuildError(code, "contract differs from the declared V1 boundary")


def _digest(value: object) -> str:
    if (
        not isinstance(value, str)
        or len(value) != 64
        or any(c not in "0123456789abcdef" for c in value)
    ):
        raise QcewBuildError("contract_sha256", str(value))
    return value


def verify_contract(contract: dict[str, Any]) -> None:
    """Pin selection, null interpretation, units, bounds, and lack of mechanics authority."""
    _equal(
        sorted(contract),
        sorted(
            ["meta", "classifications", "bounds", "source", "artifact", "semantics", "authority"]
        ),
        "contract_shape",
    )
    _equal(
        contract["meta"],
        {
            "contract": "QcewCountySectorsV1",
            "version": 1,
            "issue": "PER-29",
            "parent": "PER-10",
            "delivery": "observed-foundation-artifact-only",
        },
        "contract_meta",
    )
    _equal(
        contract["classifications"],
        {
            "source_cells_values_titles_disclosure_and_vintage": "Observed",
            "coverage_counts_and_semantic_digest": "Derived",
            "canonical_format_ordering_and_dispositions": "Designed",
        },
        "contract_classifications",
    )
    _equal(
        contract["bounds"],
        {
            "contract_bytes": 65_536,
            "artifact_bytes": MAX_ARTIFACT_BYTES,
            "artifact_uncompressed_bytes": MAX_ARTIFACT_BYTES,
            "artifact_rows": builder.MAX_ROWS,
            "integer_maximum": builder.MAX_I64,
        },
        "contract_bounds",
    )
    _equal(
        contract["source"],
        {
            "manifest": SOURCE_MANIFEST_PATH,
            "manifest_sha256": "048c02b5890115e655e0a61553472adf2cbf8ef5c016731c0ffc0bfd0f2f667e",
            "files": 83,
            "year": "2024",
            "qtr": "A",
            "size_code": "0",
            "own_code": "5",
            "agglvl_code": "74",
            "admission": "annual_avg_estabs_count-positive",
            "sector_codes": list(builder.SECTOR_CODES),
            "archive_url": "https://data.bls.gov/cew/data/files/2024/csv/2024_annual_by_area.zip",
            "aggregation_reference": "https://www.bls.gov/cew/classifications/aggregation/agg-level-titles.htm",
            "disclosure_reference": "https://www.bls.gov/cew/questions-and-answers.htm",
        },
        "contract_source",
    )
    artifact = contract["artifact"]
    if not isinstance(artifact, dict):
        raise QcewBuildError("contract_artifact", "mapping required")
    _equal(
        artifact,
        {
            "name": builder.ARTIFACT_NAME,
            "path": builder.ARTIFACT_PATH,
            "format": "csv.gz",
            "compression": "gzip-mtime-0",
            "ordering": "county_geoid-then-sector_code-ascending",
            "columns": list(builder.COLUMNS),
            "semantic_domain": "babylon.qcew-county-sectors.v1",
            "semantic_encoding": "domain-NUL-then-compact-JSON-columns-LF-then-typed-JSON-row-LF",
            "sha256": _digest(artifact.get("sha256")),
            "semantic_sha256": _digest(artifact.get("semantic_sha256")),
            **EXPECTED_CENSUS,
        },
        "contract_artifact",
    )
    _equal(contract["semantics"], EXPECTED_SEMANTICS, "contract_semantics")
    _equal(
        contract["authority"],
        {
            "reference_only": True,
            "planned_graph_node": "ORGANIZATION-with-OrgKind-BUSINESS",
            "planned_membership": "native-ECONOMIC_SECTOR-hyperedge",
            "unclassified_membership": "unspecified-no-invented-classified-sector",
            "mechanics_consumers": [],
            "excluded": [
                "runtime-hydration",
                "worker-allocation",
                "labor-hours",
                "recipes",
                "physical-output",
                "reserve-army",
                "class-movement",
                "money-flows",
                "PER-29-completion",
            ],
            "rulings": ["ADR196", "ADR250-R3"],
        },
        "contract_authority",
    )


def verify_source_manifest(contract: dict[str, Any], root: Path) -> dict[str, str]:
    """Reuse the existing bounded manifest parser, checking its exact content pin."""
    path = root / SOURCE_MANIFEST_PATH
    entries = load_source_manifest(path)
    actual = hashlib.sha256(_bounded_bytes(path, 131_072)).hexdigest()
    if actual != contract["source"]["manifest_sha256"]:
        raise QcewBuildError("source_manifest_sha256", actual)
    return {entry["file"]: entry["sha256"] for entry in entries}


def _integer(value: str) -> int:
    if ASCII_DIGITS.fullmatch(value) is None or len(value) > 19:
        raise QcewBuildError("artifact_value", value)
    result = int(value)
    if result > builder.MAX_I64 or str(result) != value:
        raise QcewBuildError("artifact_value", value)
    return result


def _read_row(row: list[str], pins: dict[str, str]) -> builder.SectorRow:
    if len(row) != len(builder.COLUMNS):
        raise QcewBuildError("artifact_row_shape", str(len(row)))
    geoid, sector, title, disposition, disclosure, estabs, jobs, payroll, wage, source, digest = row
    if geoid not in MICHIGAN_COUNTY_GEOIDS or sector not in builder.SECTOR_CODES:
        raise QcewBuildError("artifact_identity", f"{geoid}/{sector}")
    if not title or len(title) > 256 or not title.isprintable():
        raise QcewBuildError("artifact_title", f"{geoid}/{sector}")
    if disposition != ("unclassified" if sector == "99" else "classified"):
        raise QcewBuildError("artifact_disposition", f"{geoid}/{sector}")
    if disclosure not in {"", "N"} or (disclosure == "N" and (jobs, payroll, wage) != ("", "", "")):
        raise QcewBuildError("artifact_disclosure", f"{geoid}/{sector}")
    establishment_count = _integer(estabs)
    if establishment_count == 0:
        raise QcewBuildError("artifact_empty_cell", f"{geoid}/{sector}")
    match = COUNTY_FILE_RE.fullmatch(source)
    if match is None or match.group(1) != geoid or pins.get(source) != digest:
        raise QcewBuildError("artifact_provenance", f"{geoid}/{sector}")
    return builder.SectorRow(
        geoid,
        sector,
        title,
        disposition,
        disclosure,
        establishment_count,
        None if disclosure else _integer(jobs),
        None if disclosure else _integer(payroll),
        None if disclosure else _integer(wage),
        source,
        digest,
    )


def _read_rows(raw: bytes, pins: dict[str, str]) -> tuple[builder.SectorRow, ...]:
    if len(raw) < 10 or raw[:4] != b"\x1f\x8b\x08\x00" or raw[4:8] != b"\x00" * 4:
        raise QcewBuildError("artifact_gzip_header", "expected no filename and mtime zero")
    try:
        with gzip.GzipFile(fileobj=io.BytesIO(raw), mode="rb") as handle:
            decoded = handle.read(MAX_ARTIFACT_BYTES + 1)
        if len(decoded) > MAX_ARTIFACT_BYTES:
            raise QcewBuildError("artifact_uncompressed_size", str(len(decoded)))
        reader = csv.reader(io.StringIO(decoded.decode("utf-8"), newline=""), strict=True)
        if tuple(next(reader, ())) != builder.COLUMNS:
            raise QcewBuildError("artifact_columns", "unexpected CSV header")
        rows: list[builder.SectorRow] = []
        for values in reader:
            if len(rows) >= builder.MAX_ROWS:
                raise QcewBuildError("artifact_rows", str(len(rows)))
            rows.append(_read_row(values, pins))
    except (OSError, EOFError, UnicodeDecodeError, csv.Error) as error:
        raise QcewBuildError("artifact_decode", "invalid bounded gzip/CSV") from error
    return tuple(rows)


def verify_artifact(contract: dict[str, Any], root: Path) -> tuple[builder.SectorRow, ...]:
    """Validate byte pin, typed rows, source lineage, coverage, and semantic digest."""
    spec = contract["artifact"]
    pins = verify_source_manifest(contract, root)
    # Fixed path, even when called directly with a malformed contract.
    raw = _bounded_bytes(root / builder.ARTIFACT_PATH, MAX_ARTIFACT_BYTES)
    if hashlib.sha256(raw).hexdigest() != spec["sha256"]:
        raise QcewBuildError("artifact_sha256", "checked bytes differ")
    rows = _read_rows(raw, pins)
    keys = [(row.county_geoid, row.sector_code) for row in rows]
    if keys != sorted(set(keys)):
        raise QcewBuildError("artifact_order", "unordered or duplicate county-sector cell")
    stats = asdict(builder.artifact_stats(rows, raw))
    if any(stats[key] != spec[key] for key in EXPECTED_CENSUS):
        raise QcewBuildError("artifact_census", str(stats))
    if stats["semantic_sha256"] != spec["semantic_sha256"]:
        raise QcewBuildError("artifact_semantic_sha256", str(stats["semantic_sha256"]))
    return rows


def verify_artifact_manifest(contract: dict[str, Any], path: Path) -> None:
    """Tripwire the second-order entry without regenerating the shared registry."""
    document = _load_mapping(path, 262_144)
    entries = document.get("artifacts")
    if not isinstance(entries, list) or any(not isinstance(entry, dict) for entry in entries):
        raise QcewBuildError("artifact_manifest_shape", "artifacts")
    matches = [entry for entry in entries if entry.get("name") == builder.ARTIFACT_NAME]
    if len(matches) != 1:
        raise QcewBuildError("artifact_manifest_pin", builder.ARTIFACT_NAME)
    entry = matches[0]
    relation = entry.get("material_relation")
    expected = {
        "name": builder.ARTIFACT_NAME,
        "format": "csv.gz",
        "source_table": None,
        "generator": "tools/make_qcew_county_sector_artifacts.py",
        "mode": "register",
        "rows": contract["artifact"]["rows"],
        "sha256": contract["artifact"]["sha256"],
        "home": builder.ARTIFACT_PATH,
        "material_relation": relation,
    }
    if entry != expected or not isinstance(relation, str) or "Part of PER-29" not in relation:
        raise QcewBuildError("artifact_manifest_pin", builder.ARTIFACT_NAME)


def verify_acquisition(contract: dict[str, Any], root: Path, source_dir: Path) -> None:
    """Explicit source replay writes only a uniquely named temporary directory."""
    verify_source_manifest(contract, root)
    with TemporaryDirectory(prefix="babylon-qcew-sectors-verification-") as temporary:
        stats = builder.build(
            source_dir=source_dir,
            source_manifest=root / SOURCE_MANIFEST_PATH,
            out_path=Path(temporary) / "sectors.csv.gz",
        )
        spec = contract["artifact"]
        if (stats.sha256, stats.semantic_sha256) != (spec["sha256"], spec["semantic_sha256"]):
            raise QcewBuildError(
                "artifact_regeneration", "source replay differs from checked artifact"
            )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument(
        "--source-dir", type=Path, help="explicitly replay pinned source in temporary output"
    )
    args = parser.parse_args(argv)
    contract = load_contract(args.repo_root / CONTRACT_PATH)
    verify_contract(contract)
    rows = verify_artifact(contract, args.repo_root)
    verify_artifact_manifest(contract, args.repo_root / "data-artifacts.yaml")
    if args.source_dir is not None:
        verify_acquisition(contract, args.repo_root, args.source_dir)
    print(
        f"QcewCountySectorsV1 verified: {len(rows)} county-sector rows, 83 counties, 416 rows with suppressed metrics"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
