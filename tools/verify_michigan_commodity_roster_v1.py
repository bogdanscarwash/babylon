#!/usr/bin/env python3
"""Verify the checked commodity roster without consulting local raw acquisition.

An explicit --source-dir additionally checks the 83 source hashes and rebuilds
into temporary output. Verification never rewrites a checked artifact.
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import io
import json
from dataclasses import asdict, fields
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any, Final

import make_michigan_commodity_roster as builder
import make_qcew_county_economics_artifacts as county
import make_qcew_county_sector_artifacts as sectors
import verify_qcew_county_sectors_v1 as sector_verifier
from make_qcew_county_economics_artifacts import QcewBuildError

CONTRACT_PATH: Final = "contracts/michigan_commodity_roster_v1.yaml"
SOURCE_MANIFEST_PATH: Final = "tools/qcew_county_economics_v1_source_manifest.json"
SOURCE_MANIFEST_SHA256: Final = "048c02b5890115e655e0a61553472adf2cbf8ef5c016731c0ffc0bfd0f2f667e"
CONTEXT_SHA256: Final = "1cac80bee20c086be2e1f268643b0caab5d8d63030251817a9f149123755d71a"
EXPECTED_COVERAGE: Final = {
    "actors": 397,
    "producers": 231,
    "wholesalers": 83,
    "retailers": 83,
    "counties": 83,
    "excluded_cohorts": 5,
}


def _equal(actual: object, expected: object, code: str) -> None:
    if builder.canonical_json({"value": actual}) != builder.canonical_json({"value": expected}):
        raise QcewBuildError(code, "value differs from the declared V1 boundary")


def _bounded_bytes(path: Path, maximum: int) -> bytes:
    try:
        with path.open("rb") as handle:
            raw = handle.read(maximum + 1)
    except OSError as error:
        raise QcewBuildError("file_read", str(path)) from error
    if len(raw) > maximum:
        raise QcewBuildError("file_size", str(path))
    return raw


def _unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise QcewBuildError("artifact_duplicate_key", key)
        result[key] = value
    return result


def _invalid_constant(value: str) -> None:
    raise QcewBuildError("artifact_nonfinite", value)


def decode_artifact(raw: bytes) -> dict[str, Any]:
    """Bound decompression, reject duplicate keys, and require canonical JSON bytes."""
    if len(raw) > builder.MAX_ARTIFACT_BYTES:
        raise QcewBuildError("artifact_size", str(len(raw)))
    if len(raw) < 10 or raw[:4] != b"\x1f\x8b\x08\x00" or raw[4:8] != b"\0" * 4:
        raise QcewBuildError("artifact_gzip_header", "expected no filename and mtime zero")
    try:
        with gzip.GzipFile(fileobj=io.BytesIO(raw), mode="rb") as handle:
            decoded = handle.read(builder.MAX_DECODED_BYTES + 1)
        if len(decoded) > builder.MAX_DECODED_BYTES:
            raise QcewBuildError("artifact_uncompressed_size", str(len(decoded)))
        document = json.loads(
            decoded, object_pairs_hook=_unique_object, parse_constant=_invalid_constant
        )
    except (OSError, EOFError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise QcewBuildError("artifact_decode", "invalid bounded gzip/JSON") from error
    if not isinstance(document, dict) or builder.canonical_json(document) != decoded:
        raise QcewBuildError("artifact_encoding", "canonical JSON object required")
    return document


def _industry(value: object) -> builder.IndustryRow:
    if not isinstance(value, dict) or set(value) != {
        field.name for field in fields(builder.IndustryRow)
    }:
        raise QcewBuildError("artifact_industry_shape", "unexpected fields")
    strings = (
        "county_geoid",
        "sector_code",
        "industry_code",
        "agglvl_code",
        "industry_title",
        "disclosure_code",
    )
    if any(not isinstance(value[key], str) for key in strings):
        raise QcewBuildError("artifact_industry_shape", "string required")
    geoid, code = value["county_geoid"], value["industry_code"]
    if (
        geoid not in county.MICHIGAN_COUNTY_GEOIDS
        or builder.detail_sector(code, value["agglvl_code"]) != value["sector_code"]
    ):
        raise QcewBuildError("artifact_industry_identity", f"{geoid}/{code}")
    title, disclosure = value["industry_title"], value["disclosure_code"]
    if not title or len(title) > 256 or not title.isprintable():
        raise QcewBuildError("artifact_industry_title", f"{geoid}/{code}")
    if disclosure not in {"", "N"}:
        raise QcewBuildError("artifact_disclosure", f"{geoid}/{code}")
    for key in sectors.METRIC_COLUMNS:
        metric = value[key]
        if key != "annual_avg_estabs_count" and disclosure == "N":
            if metric is not None:
                raise QcewBuildError("artifact_suppression", f"{geoid}/{code}/{key}")
        elif type(metric) is not int or not 0 <= metric <= sectors.MAX_I64:
            raise QcewBuildError("artifact_value", f"{geoid}/{code}/{key}")
    if value["annual_avg_estabs_count"] == 0:
        raise QcewBuildError("artifact_empty_industry", f"{geoid}/{code}")
    return builder.IndustryRow(**value)


def verify_document(document: dict[str, Any], root: Path) -> None:
    """Recompute eligibility from typed evidence and the unchanged sector context."""
    _equal(
        sorted(document),
        sorted(
            [
                "schema",
                "vintage",
                "source_manifest_sha256",
                "sector_context",
                "sources",
                "industries",
                "actors",
                "excluded_cohorts",
            ]
        ),
        "artifact_shape",
    )
    _equal(document["schema"], builder.SCHEMA, "artifact_schema")
    _equal(document["vintage"], 2024, "artifact_vintage")
    manifest_path = root / SOURCE_MANIFEST_PATH
    manifest = _bounded_bytes(manifest_path, county.MAX_SOURCE_MANIFEST_BYTES)
    _equal(hashlib.sha256(manifest).hexdigest(), SOURCE_MANIFEST_SHA256, "source_manifest_sha256")
    _equal(document["source_manifest_sha256"], SOURCE_MANIFEST_SHA256, "artifact_source_manifest")
    entries = county.load_source_manifest(manifest_path)
    _equal(
        document["sources"],
        [
            {"county_geoid": geoid, **entry}
            for geoid, entry in zip(county.MICHIGAN_COUNTY_GEOIDS, entries, strict=True)
        ],
        "artifact_provenance",
    )
    _equal(
        document["sector_context"],
        {"path": builder.CONTEXT_PATH, "sha256": CONTEXT_SHA256},
        "artifact_context",
    )
    context_contract = sector_verifier.load_contract(root / sector_verifier.CONTRACT_PATH)
    sector_verifier.verify_contract(context_contract)
    context = sector_verifier.verify_artifact(context_contract, root)
    values = document["industries"]
    if not isinstance(values, list) or not 0 < len(values) <= builder.MAX_INDUSTRIES:
        raise QcewBuildError("artifact_industry_rows", "bounded nonempty list required")
    industries = tuple(_industry(value) for value in values)
    identities = [(row.county_geoid, row.industry_code) for row in industries]
    if identities != sorted(set(identities)):
        raise QcewBuildError("artifact_industry_order", "unordered or duplicate source cell")
    actors, excluded = [], []
    for geoid in county.MICHIGAN_COUNTY_GEOIDS:
        selected, omitted = builder.select_actors(
            geoid,
            (row.sector_code for row in context if row.county_geoid == geoid),
            [row for row in industries if row.county_geoid == geoid],
        )
        actors.extend(asdict(actor) for actor in selected)
        excluded.extend(asdict(row) for row in omitted)
    _equal(document["actors"], actors, "artifact_actor_selection")
    _equal(document["excluded_cohorts"], excluded, "artifact_excluded_selection")


def load_contract(path: Path) -> dict[str, Any]:
    """Reuse the existing bounded, duplicate-rejecting YAML contract reader."""
    return sector_verifier.load_contract(path)


def verify_contract(contract: dict[str, Any]) -> None:
    _equal(
        sorted(contract),
        sorted(
            [
                "meta",
                "classifications",
                "source",
                "context",
                "selection",
                "families",
                "artifact",
                "bounds",
                "semantics",
                "authority",
            ]
        ),
        "contract_shape",
    )
    _equal(
        contract["meta"],
        {
            "contract": builder.SCHEMA,
            "version": 1,
            "issue": "PER-29",
            "parent": "PER-10",
            "delivery": "commodity-eligibility-artifact",
        },
        "contract_meta",
    )
    _equal(
        contract["classifications"],
        {
            "source_fields": "Observed",
            "family_catalogue_and_selection": "Designed",
            "coverage_and_digests": "Derived",
        },
        "contract_classifications",
    )
    _equal(
        contract["source"],
        {
            "manifest": SOURCE_MANIFEST_PATH,
            "manifest_sha256": SOURCE_MANIFEST_SHA256,
            "files": 83,
            "year": "2024",
            "qtr": "A",
            "own_code": "5",
            "size_code": "0",
            "detail_aggregation_levels": ["75", "76"],
            "admission": "positive-establishments",
        },
        "contract_source",
    )
    _equal(
        contract["context"],
        {
            "path": builder.CONTEXT_PATH,
            "sha256": CONTEXT_SHA256,
            "rows": 1603,
            "sector_codes": list(sectors.SECTOR_CODES),
        },
        "contract_context",
    )
    _equal(
        contract["selection"],
        {
            "actor_identity": ["county_geoid", "sector_code"],
            "commodity_sectors": list(builder.COMMODITY_SECTORS),
            "primary": "greatest-positive-establishments-then-exact-industry-code-ascending",
            "mining": "choose-211-or-212-first-then-four-digit-child-of-212",
            "support_only_excluded": ["115", "213", "425"],
            "physical_wholesale": ["423", "424"],
            "unmapped_primary": "refuse",
            "alternatives": "every-positive-exact-catalogue-industry-row",
            "merchant_family": None,
        },
        "contract_selection",
    )
    _equal(contract["families"], builder.FAMILY_BY_INDUSTRY, "contract_families")
    _equal(
        contract["bounds"],
        {
            "artifact_bytes": builder.MAX_ARTIFACT_BYTES,
            "artifact_uncompressed_bytes": builder.MAX_DECODED_BYTES,
            "industry_rows": builder.MAX_INDUSTRIES,
            "actors": builder.MAX_ACTORS,
            "integer_maximum": sectors.MAX_I64,
        },
        "contract_bounds",
    )
    _equal(
        contract["semantics"],
        {
            "establishments": "annual-average-establishments-not-operating-capacity",
            "employment": "annual-average-jobs-not-distinct-people-or-modeled-workers",
            "total_annual_wages": "calendar-year-total-USD-not-engine-money",
            "annual_avg_wkly_wage": "annual-average-USD-per-employee-per-week-not-median",
            "suppression": "N-retains-establishments-other-three-metrics-null",
            "disclosed_zero": "exact-observed-zero",
            "absent_row": "absent-not-zero",
            "nested_industries": "preserved-never-added-together",
            "provenance": "county-geoid-joins-one-exact-source-filename-and-sha256",
        },
        "contract_semantics",
    )
    _equal(
        contract["authority"],
        {
            "reference_only": True,
            "family_meaning": "Designed-representative-goods-not-observed-products-or-factories",
            "excluded": [
                "recipes",
                "operating-quantities",
                "workforce-allocation",
                "supplier-selection",
                "routes",
                "physical-capacity",
                "payments",
                "services",
                "PER-29-completion",
            ],
        },
        "contract_authority",
    )
    artifact = contract["artifact"]
    if not isinstance(artifact, dict):
        raise QcewBuildError("contract_artifact", "mapping required")
    keys = {
        "name",
        "path",
        "format",
        "compression",
        "ordering",
        "semantic_domain",
        "sha256",
        "semantic_sha256",
        "industries",
        "suppressed_industries",
        *EXPECTED_COVERAGE,
    }
    _equal(sorted(artifact), sorted(keys), "contract_artifact_shape")
    for key, value in {
        "name": builder.ARTIFACT_NAME,
        "path": builder.ARTIFACT_PATH,
        "format": "json.gz",
        "compression": "gzip-mtime-0",
        "ordering": "county-geoid-then-exact-code-ascending",
        "semantic_domain": "babylon.michigan-commodity-roster.v1",
        **EXPECTED_COVERAGE,
    }.items():
        _equal(artifact[key], value, "contract_artifact")
    for key in ("sha256", "semantic_sha256"):
        value = artifact[key]
        if (
            not isinstance(value, str)
            or len(value) != 64
            or any(char not in "0123456789abcdef" for char in value)
        ):
            raise QcewBuildError("contract_digest", key)
    for key in ("industries", "suppressed_industries"):
        if type(artifact[key]) is not int or not 0 < artifact[key] <= builder.MAX_INDUSTRIES:
            raise QcewBuildError("contract_census", key)


def verify_artifact(contract: dict[str, Any], root: Path) -> builder.ArtifactStats:
    raw = _bounded_bytes(root / builder.ARTIFACT_PATH, builder.MAX_ARTIFACT_BYTES)
    _equal(hashlib.sha256(raw).hexdigest(), contract["artifact"]["sha256"], "artifact_sha256")
    document = decode_artifact(raw)
    verify_document(document, root)
    stats = builder.artifact_stats(document, raw)
    for key, value in asdict(stats).items():
        _equal(value, contract["artifact"][key], "artifact_census_or_digest")
    return stats


def verify_acquisition(contract: dict[str, Any], root: Path, source_dir: Path) -> None:
    with TemporaryDirectory(prefix="babylon-commodity-roster-verification-") as temporary:
        stats = builder.build(
            source_dir=source_dir,
            source_manifest=root / SOURCE_MANIFEST_PATH,
            context_path=root / builder.CONTEXT_PATH,
            out_path=Path(temporary) / "roster.json.gz",
        )
        _equal(stats.sha256, contract["artifact"]["sha256"], "artifact_regeneration")
        _equal(
            stats.semantic_sha256, contract["artifact"]["semantic_sha256"], "artifact_regeneration"
        )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument(
        "--source-dir", type=Path, help="explicitly replay pinned source into temporary output"
    )
    args = parser.parse_args(argv)
    contract = load_contract(args.repo_root / CONTRACT_PATH)
    verify_contract(contract)
    stats = verify_artifact(contract, args.repo_root)
    if args.source_dir is not None:
        verify_acquisition(contract, args.repo_root, args.source_dir)
    print(
        f"{builder.SCHEMA} verified: {stats.actors} owners, {stats.producers} producers, {stats.industries} observed detail rows"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
