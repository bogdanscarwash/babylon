#!/usr/bin/env python3
"""Build Michigan's observed 2024 private county-sector foundation reference.

Reuses the existing 83-file acquisition manifest; no download or database writer
is involved. BLS disclosure code N preserves establishments but withholds the
other measures. Blank output metrics therefore mean unavailable, never zero.
"""

from __future__ import annotations

import argparse
import csv
import gzip
import hashlib
import io
import json
from collections.abc import Iterable, Sequence
from dataclasses import asdict, astuple, dataclass, replace
from pathlib import Path
from typing import Final

from make_qcew_county_economics_artifacts import (
    ASCII_DIGITS,
    COUNTY_FILE_RE,
    EXPECTED_SOURCE_COLUMNS,
    MICHIGAN_COUNTY_GEOIDS,
    REPO_ROOT,
    SOURCE_MANIFEST,
    QcewBuildError,
    load_source_manifest,
    verify_source_file,
)

ARTIFACT_NAME: Final = "qcew_county_sectors_mi_2024"
ARTIFACT_PATH: Final = f"src/babylon/data/reference/economy/{ARTIFACT_NAME}.csv.gz"
ARTIFACT_OUT: Final = REPO_ROOT / ARTIFACT_PATH
DOMAIN: Final = b"babylon.qcew-county-sectors.v1\0"
SECTOR_CODES: Final = (
    "11",
    "21",
    "22",
    "23",
    "31-33",
    "42",
    "44-45",
    "48-49",
    "51",
    "52",
    "53",
    "54",
    "55",
    "56",
    "61",
    "62",
    "71",
    "72",
    "81",
    "99",
)
COLUMNS: Final = (
    "county_geoid",
    "sector_code",
    "sector_title",
    "sector_disposition",
    "disclosure_code",
    "annual_avg_estabs_count",
    "annual_avg_emplvl",
    "total_annual_wages",
    "annual_avg_wkly_wage",
    "source_file",
    "source_sha256",
)
METRIC_COLUMNS: Final = COLUMNS[5:9]
MAX_I64: Final = 2**63 - 1
MAX_ROWS: Final = len(MICHIGAN_COUNTY_GEOIDS) * len(SECTOR_CODES)


@dataclass(frozen=True)
class SectorRow:
    """Observed cell; a missing metric is disclosure, not an employment outcome."""

    county_geoid: str
    sector_code: str
    sector_title: str
    sector_disposition: str
    disclosure_code: str
    annual_avg_estabs_count: int
    annual_avg_emplvl: int | None
    total_annual_wages: int | None
    annual_avg_wkly_wage: int | None
    source_file: str = ""
    source_sha256: str = ""


@dataclass(frozen=True)
class ArtifactStats:
    """Byte identity, semantic identity and coverage, with no suppressed totals."""

    rows: int
    counties: int
    disclosed_rows: int
    suppressed_rows: int
    classified_rows: int
    unclassified_rows: int
    absent_cells: int
    sha256: str
    semantic_sha256: str


def _integer(value: str | None, identity: str) -> int:
    if value is None or ASCII_DIGITS.fullmatch(value) is None or len(value) > 19:
        raise QcewBuildError("source_value", identity)
    result = int(value)
    if result > MAX_I64:
        raise QcewBuildError("source_value", identity)
    return result


def _canonical_row(file_geoid: str, row: dict[str, str]) -> SectorRow:
    if row.get("area_fips") != file_geoid:
        raise QcewBuildError("source_geoid", file_geoid)
    if (row.get("year"), row.get("qtr"), row.get("size_code")) != ("2024", "A", "0"):
        raise QcewBuildError("source_row_identity", file_geoid)
    sector = row.get("industry_code", "")
    if sector not in SECTOR_CODES:
        raise QcewBuildError("source_sector", sector)
    title = row.get("industry_title", "")
    if not title or len(title) > 256 or not title.isprintable():
        raise QcewBuildError("source_title", sector)
    disclosure = row.get("disclosure_code", "")
    if disclosure not in {"", "N"}:
        raise QcewBuildError("source_disclosure", f"{file_geoid}/{sector}")
    estabs, jobs, payroll, wage = (
        _integer(row.get(column), f"{file_geoid}/{sector}/{column}") for column in METRIC_COLUMNS
    )
    if disclosure == "N" and (jobs, payroll, wage) != (0, 0, 0):
        raise QcewBuildError("source_suppressed_value", f"{file_geoid}/{sector}")
    return SectorRow(
        file_geoid,
        sector,
        title,
        "unclassified" if sector == "99" else "classified",
        disclosure,
        estabs,
        None if disclosure else jobs,
        None if disclosure else payroll,
        None if disclosure else wage,
    )


def canonicalize_sector_rows(
    file_geoid: str,
    fieldnames: Sequence[str] | None,
    rows: Iterable[dict[str, str]],
) -> tuple[SectorRow, ...]:
    """Select private sector cells without imputing, truncating, or rolling up detail."""
    if tuple(fieldnames or ()) != EXPECTED_SOURCE_COLUMNS:
        raise QcewBuildError("source_schema", repr(tuple(fieldnames or ())))
    if file_geoid not in MICHIGAN_COUNTY_GEOIDS:
        raise QcewBuildError("source_geoid", file_geoid)
    selected: dict[str, SectorRow] = {}
    seen: set[str] = set()
    for row in rows:
        if row.get("agglvl_code") != "74" or row.get("own_code") != "5":
            continue
        if set(row) != set(EXPECTED_SOURCE_COLUMNS) or any(value is None for value in row.values()):
            raise QcewBuildError("source_row_shape", file_geoid)
        canonical = _canonical_row(file_geoid, row)
        if canonical.sector_code in seen:
            raise QcewBuildError("source_duplicate_sector", f"{file_geoid}/{canonical.sector_code}")
        seen.add(canonical.sector_code)
        if canonical.annual_avg_estabs_count > 0:
            selected[canonical.sector_code] = canonical
    if not selected:
        raise QcewBuildError("source_selection", file_geoid)
    return tuple(selected[key] for key in sorted(selected))


def semantic_sha256(rows: Sequence[SectorRow]) -> str:
    """Hash the fixed schema and typed rows, including nulls and exact source lineage."""
    digest = hashlib.sha256(DOMAIN)
    digest.update(json.dumps(COLUMNS, separators=(",", ":")).encode("ascii") + b"\n")
    for row in rows:
        digest.update(
            json.dumps(
                astuple(row), ensure_ascii=True, separators=(",", ":"), allow_nan=False
            ).encode("ascii")
            + b"\n"
        )
    return digest.hexdigest()


def artifact_stats(rows: Sequence[SectorRow], raw: bytes) -> ArtifactStats:
    """Count admitted source cells, not hypothetical enterprises or workers."""
    suppressed = sum(row.disclosure_code == "N" for row in rows)
    unclassified = sum(row.sector_disposition == "unclassified" for row in rows)
    return ArtifactStats(
        len(rows),
        len({row.county_geoid for row in rows}),
        len(rows) - suppressed,
        suppressed,
        len(rows) - unclassified,
        unclassified,
        MAX_ROWS - len(rows),
        hashlib.sha256(raw).hexdigest(),
        semantic_sha256(rows),
    )


def _read_county(source_dir: Path, entry: dict[str, str]) -> tuple[SectorRow, ...]:
    match = COUNTY_FILE_RE.fullmatch(entry["file"])
    if match is None:
        raise QcewBuildError("source_manifest_file", entry["file"])
    source = source_dir / entry["file"]
    verify_source_file(source, entry["sha256"])
    try:
        with source.open(newline="", encoding="utf-8") as handle:
            reader = csv.DictReader(handle)
            rows = canonicalize_sector_rows(match.group(1), reader.fieldnames, reader)
    except (OSError, csv.Error, UnicodeDecodeError) as error:
        raise QcewBuildError("source_csv", str(source)) from error
    return tuple(
        replace(row, source_file=entry["file"], source_sha256=entry["sha256"]) for row in rows
    )


def build(
    *,
    source_dir: Path,
    out_path: Path = ARTIFACT_OUT,
    source_manifest: Path = SOURCE_MANIFEST,
) -> ArtifactStats:
    """Verify all source files and materialize only the named sector output."""
    entries = load_source_manifest(source_manifest)
    rows = tuple(row for entry in entries for row in _read_county(source_dir, entry))
    if len(rows) > MAX_ROWS:
        raise QcewBuildError("artifact_rows", str(len(rows)))
    if (
        out_path.resolve().is_relative_to(source_dir.resolve())
        or out_path.resolve() == source_manifest.resolve()
    ):
        raise QcewBuildError("output_source_overlap", str(out_path))
    buffer = io.BytesIO()
    with (
        gzip.GzipFile(filename="", fileobj=buffer, mode="wb", compresslevel=9, mtime=0) as binary,
        io.TextIOWrapper(binary, encoding="utf-8", newline="") as text,
    ):
        writer = csv.writer(text, lineterminator="\n")
        writer.writerow(COLUMNS)
        writer.writerows(astuple(row) for row in rows)
    raw = buffer.getvalue()
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_bytes(raw)
    return artifact_stats(rows, raw)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-dir", type=Path, required=True)
    parser.add_argument("--out", type=Path, default=ARTIFACT_OUT)
    parser.add_argument("--source-manifest", type=Path, default=SOURCE_MANIFEST)
    args = parser.parse_args(argv)
    stats = build(
        source_dir=args.source_dir, out_path=args.out, source_manifest=args.source_manifest
    )
    print(json.dumps(asdict(stats), indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
