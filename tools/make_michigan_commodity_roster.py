#!/usr/bin/env python3
"""Build Michigan commodity eligibility from the pinned 2024 private QCEW files.

Observed industry presence supports Designed representative family choices. This
artifact contains no recipes, operating quantities, supplier choices, or routes.
The existing twenty-sector reference remains the complete observed context.
"""

from __future__ import annotations

import argparse
import csv
import gzip
import hashlib
import io
import json
from collections.abc import Iterable, Sequence
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Final

import make_qcew_county_economics_artifacts as county
import make_qcew_county_sector_artifacts as sectors
from make_qcew_county_economics_artifacts import QcewBuildError

REPO_ROOT: Final = Path(__file__).resolve().parents[1]
ARTIFACT_NAME: Final = "michigan_commodity_roster_mi_2024"
ARTIFACT_PATH: Final = f"src/babylon/data/reference/economy/{ARTIFACT_NAME}.json.gz"
CONTEXT_PATH: Final = sectors.ARTIFACT_PATH
SCHEMA: Final = "MichiganCommodityRosterV1"
DOMAIN: Final = b"babylon.michigan-commodity-roster.v1\0"
COMMODITY_SECTORS: Final = ("11", "21", "31-33", "42", "44-45")
MAX_ARTIFACT_BYTES: Final = 1_048_576
MAX_DECODED_BYTES: Final = 4_194_304
MAX_INDUSTRIES: Final = 8_000
MAX_ACTORS: Final = 415
# Keys are exact observed industry codes, not prefixes or inferred products.
FAMILY_BY_INDUSTRY: Final = {
    "111": "grain",
    "112": "animal_products",
    "113": "logs",
    "211": "hydrocarbon_feedstock",
    "2122": "metal_ore",
    "2123": "mineral_feedstock",
    "311": "prepared_food",
    "312": "packaged_beverages",
    "321": "wood_products",
    "322": "paper_packaging",
    "325": "industrial_chemicals",
    "331": "metal_stock",
    "332": "metal_parts",
    "333": "machinery",
    "334": "electrical_goods",
    "339": "household_wares",
}


@dataclass(frozen=True)
class IndustryRow:
    """One positive observed industry cell; nested rows are never added together."""

    county_geoid: str
    sector_code: str
    industry_code: str
    agglvl_code: str
    industry_title: str
    disclosure_code: str
    annual_avg_estabs_count: int
    annual_avg_emplvl: int | None
    total_annual_wages: int | None
    annual_avg_wkly_wage: int | None


@dataclass(frozen=True)
class Actor:
    """A county-sector owner eligible for subsequent authored circuit content."""

    county_geoid: str
    sector_code: str
    role: str
    primary_family: str | None
    eligible_families: tuple[str, ...]


@dataclass(frozen=True)
class ExcludedCohort:
    """An observed cohort retained in context without commodity activation."""

    county_geoid: str
    sector_code: str
    reason: str


@dataclass(frozen=True)
class CountySelection:
    """Immutable result before county source identity is attached to the artifact."""

    industries: tuple[IndustryRow, ...]
    actors: tuple[Actor, ...]
    excluded_cohorts: tuple[ExcludedCohort, ...]


@dataclass(frozen=True)
class ArtifactStats:
    """Derived coverage and identity; these counts are not economic quantities."""

    actors: int
    producers: int
    wholesalers: int
    retailers: int
    counties: int
    industries: int
    suppressed_industries: int
    excluded_cohorts: int
    sha256: str
    semantic_sha256: str


def detail_sector(code: str, level: str) -> str | None:
    """Select productive/support comparison rows and physical merchant evidence."""
    if level == "76" and code in {"2121", "2122", "2123"}:
        return "21"
    if level != "75" or len(code) != 3 or county.ASCII_DIGITS.fullmatch(code) is None:
        return None
    if code in {"111", "112", "113", "114", "115"}:
        return "11"
    if code in {"211", "212", "213"}:
        return "21"
    if code.startswith(("31", "32", "33")):
        return "31-33"
    if code in {"423", "424", "425"}:
        return "42"
    if code.startswith(("44", "45")):
        return "44-45"
    return None


def _integer(value: str | None, identity: str) -> int:
    if value is None or county.ASCII_DIGITS.fullmatch(value) is None or len(value) > 19:
        raise QcewBuildError("source_value", identity)
    result = int(value)
    if result > sectors.MAX_I64:
        raise QcewBuildError("source_value", identity)
    return result


def _industry_row(geoid: str, sector: str, row: dict[str, str]) -> IndustryRow:
    code = row["industry_code"]
    identity = f"{geoid}/{code}"
    if row["area_fips"] != geoid:
        raise QcewBuildError("source_geoid", identity)
    if (row["year"], row["qtr"], row["size_code"]) != ("2024", "A", "0"):
        raise QcewBuildError("source_row_identity", identity)
    title, disclosure = row["industry_title"], row["disclosure_code"]
    if not title or len(title) > 256 or not title.isprintable():
        raise QcewBuildError("source_title", identity)
    if disclosure not in {"", "N"}:
        raise QcewBuildError("source_disclosure", identity)
    estabs, jobs, payroll, wage = (
        _integer(row[column], f"{identity}/{column}") for column in sectors.METRIC_COLUMNS
    )
    if disclosure == "N" and (jobs, payroll, wage) != (0, 0, 0):
        raise QcewBuildError("source_suppressed_value", identity)
    return IndustryRow(
        geoid,
        sector,
        code,
        row["agglvl_code"],
        title,
        disclosure,
        estabs,
        None if disclosure else jobs,
        None if disclosure else payroll,
        None if disclosure else wage,
    )


def _leading(rows: Sequence[IndustryRow], identity: str) -> IndustryRow:
    if not rows:
        raise QcewBuildError("source_missing_detail", identity)
    return min(rows, key=lambda row: (-row.annual_avg_estabs_count, row.industry_code))


def select_actors(
    geoid: str, sector_codes: Iterable[str], industries: Sequence[IndustryRow]
) -> tuple[tuple[Actor, ...], tuple[ExcludedCohort, ...]]:
    """Choose representatives once, without estimating capacities or source quantities."""
    present = set(sector_codes)
    if any(row.sector_code not in present for row in industries):
        raise QcewBuildError("source_missing_sector", geoid)
    actors, excluded = [], []
    for sector in COMMODITY_SECTORS:
        if sector not in present:
            continue
        rows = [row for row in industries if row.sector_code == sector]
        identity = f"{geoid}/{sector}"
        productive = rows
        if sector == "11":
            productive = [row for row in rows if row.industry_code != "115"]
        elif sector == "21":
            productive = [row for row in rows if row.industry_code in {"211", "212"}]
        elif sector == "42":
            productive = [row for row in rows if row.industry_code in {"423", "424"}]
        if not productive and rows:
            excluded.append(ExcludedCohort(geoid, sector, "support-only"))
            continue
        primary = _leading(productive, identity)
        if sector in {"42", "44-45"}:
            actors.append(
                Actor(geoid, sector, "wholesaler" if sector == "42" else "retailer", None, ())
            )
            continue
        if primary.industry_code == "212":
            primary = _leading([row for row in rows if row.agglvl_code == "76"], identity)
        family = FAMILY_BY_INDUSTRY.get(primary.industry_code)
        if family is None:
            raise QcewBuildError("source_unmapped_primary", f"{identity}/{primary.industry_code}")
        alternatives = tuple(
            sorted(
                FAMILY_BY_INDUSTRY[row.industry_code]
                for row in rows
                if row.industry_code in FAMILY_BY_INDUSTRY
            )
        )
        actors.append(Actor(geoid, sector, "producer", family, alternatives))
    return tuple(actors), tuple(excluded)


def canonicalize_county(
    geoid: str, fieldnames: Sequence[str] | None, rows: Iterable[dict[str, str]]
) -> CountySelection:
    """Preserve positive detailed cells and compare only their exact industry level."""
    source_rows = list(rows)
    context = sectors.canonicalize_sector_rows(geoid, fieldnames, source_rows)
    selected: dict[str, IndustryRow] = {}
    seen: set[str] = set()
    for row in source_rows:
        sector = detail_sector(row.get("industry_code", ""), row.get("agglvl_code", ""))
        if row.get("own_code") != "5" or sector is None:
            continue
        if set(row) != set(county.EXPECTED_SOURCE_COLUMNS) or any(
            value is None for value in row.values()
        ):
            raise QcewBuildError("source_row_shape", geoid)
        canonical = _industry_row(geoid, sector, row)
        if canonical.industry_code in seen:
            raise QcewBuildError("source_duplicate_industry", f"{geoid}/{canonical.industry_code}")
        seen.add(canonical.industry_code)
        if canonical.annual_avg_estabs_count > 0:
            selected[canonical.industry_code] = canonical
    industries = tuple(selected[key] for key in sorted(selected))
    actors, excluded = select_actors(geoid, (row.sector_code for row in context), industries)
    return CountySelection(industries, actors, excluded)


def canonical_json(document: dict[str, Any] | CountySelection) -> bytes:
    """One canonical semantic encoding; typed nulls remain JSON null."""
    value = asdict(document) if isinstance(document, CountySelection) else document
    return (
        json.dumps(value, sort_keys=True, ensure_ascii=True, separators=(",", ":"), allow_nan=False)
        + "\n"
    ).encode("ascii")


def encode_artifact(document: dict[str, Any]) -> bytes:
    """One compressed encoder with no wall-clock or filename metadata."""
    decoded = canonical_json(document)
    if len(decoded) > MAX_DECODED_BYTES:
        raise QcewBuildError("artifact_uncompressed_size", str(len(decoded)))
    buffer = io.BytesIO()
    with gzip.GzipFile(filename="", fileobj=buffer, mode="wb", compresslevel=9, mtime=0) as handle:
        handle.write(decoded)
    raw = buffer.getvalue()
    if len(raw) > MAX_ARTIFACT_BYTES:
        raise QcewBuildError("artifact_size", str(len(raw)))
    return raw


def artifact_stats(document: dict[str, Any], raw: bytes) -> ArtifactStats:
    actors = document["actors"]
    return ArtifactStats(
        len(actors),
        sum(actor["role"] == "producer" for actor in actors),
        sum(actor["role"] == "wholesaler" for actor in actors),
        sum(actor["role"] == "retailer" for actor in actors),
        len({actor["county_geoid"] for actor in actors}),
        len(document["industries"]),
        sum(row["disclosure_code"] == "N" for row in document["industries"]),
        len(document["excluded_cohorts"]),
        hashlib.sha256(raw).hexdigest(),
        hashlib.sha256(DOMAIN + canonical_json(document)).hexdigest(),
    )


def build(
    *,
    source_dir: Path,
    out_path: Path = REPO_ROOT / ARTIFACT_PATH,
    source_manifest: Path = county.SOURCE_MANIFEST,
    context_path: Path = REPO_ROOT / CONTEXT_PATH,
) -> ArtifactStats:
    """Verify pinned acquisition before parsing; publish only a complete artifact."""
    if out_path.resolve().is_relative_to(source_dir.resolve()) or out_path.resolve() in {
        source_manifest.resolve(),
        context_path.resolve(),
    }:
        raise QcewBuildError("output_source_overlap", str(out_path))
    entries = county.load_source_manifest(source_manifest)
    sources, industries, actors, excluded = [], [], [], []
    for entry in entries:
        match = county.COUNTY_FILE_RE.fullmatch(entry["file"])
        if match is None:
            raise QcewBuildError("source_manifest_file", entry["file"])
        geoid = match.group(1)
        path = source_dir / entry["file"]
        county.verify_source_file(path, entry["sha256"])
        try:
            with path.open(newline="", encoding="utf-8") as handle:
                reader = csv.DictReader(handle)
                result = canonicalize_county(geoid, reader.fieldnames, reader)
        except (OSError, csv.Error, UnicodeDecodeError) as error:
            raise QcewBuildError("source_csv", str(path)) from error
        sources.append({"county_geoid": geoid, **entry})
        industries.extend(asdict(row) for row in result.industries)
        actors.extend(asdict(actor) for actor in result.actors)
        excluded.extend(asdict(row) for row in result.excluded_cohorts)
    if len(industries) > MAX_INDUSTRIES or len(actors) > MAX_ACTORS:
        raise QcewBuildError("artifact_rows", f"{len(industries)}/{len(actors)}")
    document = {
        "schema": SCHEMA,
        "vintage": 2024,
        "source_manifest_sha256": hashlib.sha256(source_manifest.read_bytes()).hexdigest(),
        "sector_context": {
            "path": CONTEXT_PATH,
            "sha256": hashlib.sha256(context_path.read_bytes()).hexdigest(),
        },
        "sources": sources,
        "industries": industries,
        "actors": actors,
        "excluded_cohorts": excluded,
    }
    raw = encode_artifact(document)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_bytes(raw)
    return artifact_stats(document, raw)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-dir", type=Path, required=True)
    parser.add_argument("--out", type=Path, default=REPO_ROOT / ARTIFACT_PATH)
    args = parser.parse_args(argv)
    print(json.dumps(asdict(build(source_dir=args.source_dir, out_path=args.out)), sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
