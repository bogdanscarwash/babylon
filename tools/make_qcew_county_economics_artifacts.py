#!/usr/bin/env python3
"""Build PER-319's deterministic QCEW county-economics reference artifact.

The staged BLS QCEW 2024 annual by-area CSVs are local acquisition evidence
in the ``babylon-data`` trove. Their filenames and SHA-256 digests live in
``tools/qcew_county_economics_v1_source_manifest.json`` and every digest is
verified before a CSV is parsed. The one small, immutable output is checked
in so CI and downstream work never infer authority from the local trove.
"""

from __future__ import annotations

import argparse
import csv
import gzip
import hashlib
import io
import json
import re
import sys
from collections.abc import Iterable, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Final

REPO_ROOT = Path(__file__).resolve().parents[1]
SOURCE_MANIFEST = REPO_ROOT / "tools" / "qcew_county_economics_v1_source_manifest.json"
ARTIFACT_OUT = (
    REPO_ROOT
    / "src"
    / "babylon"
    / "data"
    / "reference"
    / "economy"
    / "qcew_county_economics_mi_2024.csv.gz"
)

ARTIFACT_NAME: Final = "qcew_county_economics_mi_2024"
COLUMNS: Final = (
    "county_geoid",
    "annual_avg_estabs_count",
    "annual_avg_emplvl",
    "total_annual_wages",
    "annual_avg_wkly_wage",
)
EXPECTED_SOURCE_COLUMNS: Final = (
    "area_fips",
    "own_code",
    "industry_code",
    "agglvl_code",
    "size_code",
    "year",
    "qtr",
    "disclosure_code",
    "area_title",
    "own_title",
    "industry_title",
    "agglvl_title",
    "size_title",
    "annual_avg_estabs_count",
    "annual_avg_emplvl",
    "total_annual_wages",
    "taxable_annual_wages",
    "annual_contributions",
    "annual_avg_wkly_wage",
    "avg_annual_pay",
    "lq_disclosure_code",
    "lq_annual_avg_estabs_count",
    "lq_annual_avg_emplvl",
    "lq_total_annual_wages",
    "lq_taxable_annual_wages",
    "lq_annual_contributions",
    "lq_annual_avg_wkly_wage",
    "lq_avg_annual_pay",
    "oty_disclosure_code",
    "oty_annual_avg_estabs_count_chg",
    "oty_annual_avg_estabs_count_pct_chg",
    "oty_annual_avg_emplvl_chg",
    "oty_annual_avg_emplvl_pct_chg",
    "oty_total_annual_wages_chg",
    "oty_total_annual_wages_pct_chg",
    "oty_taxable_annual_wages_chg",
    "oty_taxable_annual_wages_pct_chg",
    "oty_annual_contributions_chg",
    "oty_annual_contributions_pct_chg",
    "oty_annual_avg_wkly_wage_chg",
    "oty_annual_avg_wkly_wage_pct_chg",
    "oty_avg_annual_pay_chg",
    "oty_avg_annual_pay_pct_chg",
)
COUNTY_FILE_RE = re.compile(r"^2024\.annual (26\d{3}) [^/\\]+ County, Michigan\.csv$")
ASCII_DIGITS = re.compile(r"^[0-9]+$")
MICHIGAN_COUNTY_GEOIDS: Final = tuple(f"{fips:05d}" for fips in range(26_001, 26_166, 2))

MAX_SOURCE_MANIFEST_BYTES: Final = 131_072
MAX_SOURCE_ENTRIES: Final = 83
MAX_SOURCE_FILE_BYTES: Final = 4_194_304
MAX_ARTIFACT_ROWS: Final = 2_048


class QcewBuildError(ValueError):
    """One typed refusal from the PER-319 artifact builder."""

    def __init__(self, code: str, detail: str) -> None:
        self.code = code
        self.detail = detail
        super().__init__(f"{code}: {detail}")


@dataclass(frozen=True)
class ArtifactStats:
    """Deterministic output evidence returned by :func:`build`."""

    rows: int
    sha256: str
    semantic_sha256: str


def _unique_json_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise QcewBuildError("source_manifest_duplicate_key", key)
        result[key] = value
    return result


def load_source_manifest(path: Path = SOURCE_MANIFEST) -> list[dict[str, str]]:
    """Load the bounded acquisition manifest with duplicate-key refusal."""
    try:
        raw = path.read_bytes()
    except OSError as error:
        raise QcewBuildError("source_manifest_read", str(path)) from error
    if len(raw) > MAX_SOURCE_MANIFEST_BYTES:
        raise QcewBuildError("source_manifest_size", str(len(raw)))
    try:
        document = json.loads(raw, object_pairs_hook=_unique_json_object)
    except (json.JSONDecodeError, UnicodeDecodeError) as error:
        raise QcewBuildError("source_manifest_json", str(path)) from error
    if not isinstance(document, dict) or document != {
        "contract": "QcewCountyEconomicsV1",
        "version": 1,
        "entries": document.get("entries") if isinstance(document, dict) else None,
    }:
        raise QcewBuildError("source_manifest_shape", "root")
    entries = document.get("entries")
    if not isinstance(entries, list) or len(entries) != MAX_SOURCE_ENTRIES:
        raise QcewBuildError("source_manifest_shape", "entries")
    result: list[dict[str, str]] = []
    files: list[str] = []
    for entry in entries:
        if not isinstance(entry, dict) or set(entry) != {"file", "sha256"}:
            raise QcewBuildError("source_manifest_entry", repr(entry))
        if not all(isinstance(entry[key], str) for key in ("file", "sha256")):
            raise QcewBuildError("source_manifest_entry", repr(entry))
        digest = entry["sha256"]
        if len(digest) != 64 or any(char not in "0123456789abcdef" for char in digest):
            raise QcewBuildError("source_manifest_sha256", entry["file"])
        files.append(entry["file"])
        result.append({"file": entry["file"], "sha256": digest})
    if files != sorted(files):
        raise QcewBuildError("source_manifest_order", repr(files[:3]))
    if len(files) != len(set(files)):
        raise QcewBuildError("source_manifest_duplicate_file", repr(files[:3]))
    geoids = []
    for name in files:
        match = COUNTY_FILE_RE.fullmatch(name)
        if match is None:
            raise QcewBuildError("source_manifest_file", name)
        geoids.append(match.group(1))
    if sorted(geoids) != sorted(MICHIGAN_COUNTY_GEOIDS):
        raise QcewBuildError("source_manifest_geoids", repr(sorted(geoids)[:3]))
    return result


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    try:
        with path.open("rb") as handle:
            for chunk in iter(lambda: handle.read(1 << 20), b""):
                digest.update(chunk)
    except OSError as error:
        raise QcewBuildError("file_read", str(path)) from error
    return digest.hexdigest()


def verify_source_file(path: Path, expected_sha256: str) -> None:
    """Verify one staged county CSV digest before it is parsed."""
    try:
        size = path.stat().st_size
    except OSError as error:
        raise QcewBuildError("source_read", str(path)) from error
    if size > MAX_SOURCE_FILE_BYTES:
        raise QcewBuildError("source_size", str(size))
    actual = _sha256(path)
    if actual != expected_sha256:
        raise QcewBuildError("source_sha256", f"expected {expected_sha256}, got {actual}")


def canonicalize_county_row(
    file_geoid: str,
    fieldnames: Sequence[str] | None,
    rows: Iterable[dict[str, str]],
) -> list[str]:
    """Validate one county CSV and return its single canonical artifact row."""
    if tuple(fieldnames or ()) != EXPECTED_SOURCE_COLUMNS:
        raise QcewBuildError("source_schema", repr(tuple(fieldnames or ())))
    selected = [
        row for row in rows if row.get("agglvl_code") == "70" and row.get("own_code") == "0"
    ]
    if len(selected) != 1:
        raise QcewBuildError("source_selection", f"{file_geoid}: {len(selected)}")
    row = selected[0]
    if row.get("area_fips") != file_geoid:
        raise QcewBuildError("source_geoid", str(row.get("area_fips")))
    if row.get("disclosure_code") not in {"", None}:
        raise QcewBuildError("source_disclosure", file_geoid)
    if (
        row.get("size_code") != "0"
        or row.get("qtr") != "A"
        or row.get("year") != "2024"
        or row.get("industry_code") != "10"
    ):
        raise QcewBuildError("source_row_identity", file_geoid)
    values = [row.get(column) for column in COLUMNS[1:]]
    if any(value is None or ASCII_DIGITS.fullmatch(value) is None for value in values):
        raise QcewBuildError("source_value", file_geoid)
    return [file_geoid, *[str(value) for value in values]]


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


def _write_gzip_csv(
    path: Path, columns: Sequence[str], rows: Sequence[Sequence[object]]
) -> ArtifactStats:
    path.parent.mkdir(parents=True, exist_ok=True)
    with (
        path.open("wb") as raw,
        gzip.GzipFile(filename="", fileobj=raw, mode="wb", compresslevel=9, mtime=0) as binary,
        io.TextIOWrapper(binary, encoding="utf-8", newline="") as text,
    ):
        writer = csv.writer(text, lineterminator="\n")
        writer.writerow(columns)
        writer.writerows(rows)
    return ArtifactStats(
        rows=len(rows),
        sha256=_sha256(path),
        semantic_sha256=_semantic_sha256(b"babylon.qcew-county-economics.v1", columns, rows),
    )


def build(
    *,
    source_dir: Path,
    out_path: Path = ARTIFACT_OUT,
    source_manifest: Path = SOURCE_MANIFEST,
) -> ArtifactStats:
    """Build the checked artifact from the 83 digest-verified county CSVs."""
    entries = load_source_manifest(source_manifest)
    canonical_rows: list[list[str]] = []
    for entry in entries:
        match = COUNTY_FILE_RE.fullmatch(entry["file"])
        if match is None:
            raise QcewBuildError("source_manifest_file", entry["file"])
        source = source_dir / entry["file"]
        verify_source_file(source, entry["sha256"])
        try:
            with source.open(newline="", encoding="utf-8") as handle:
                reader = csv.DictReader(handle)
                rows = list(reader)
                fieldnames = reader.fieldnames
        except (OSError, csv.Error, UnicodeDecodeError) as error:
            raise QcewBuildError("source_csv", str(source)) from error
        canonical_rows.append(canonicalize_county_row(match.group(1), fieldnames, rows))
    canonical_rows.sort(key=lambda row: row[0])
    geoids = [row[0] for row in canonical_rows]
    if geoids != sorted(MICHIGAN_COUNTY_GEOIDS):
        raise QcewBuildError("source_geoids", repr(geoids[:3]))
    return _write_gzip_csv(out_path, COLUMNS, canonical_rows)


def make_data_artifacts_specs() -> tuple[Any, ...]:
    """Expose the SQLite-backed generator census for the tripwire test."""
    import make_data_artifacts

    return make_data_artifacts.ARTIFACTS


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--source-dir", type=Path, required=True, help="staged county CSV directory"
    )
    parser.add_argument("--out", type=Path, help="override the artifact output path")
    parser.add_argument(
        "--source-manifest", type=Path, help="override the acquisition manifest path"
    )
    args = parser.parse_args(argv)
    stats = build(
        source_dir=args.source_dir,
        out_path=args.out if args.out is not None else ARTIFACT_OUT,
        source_manifest=args.source_manifest
        if args.source_manifest is not None
        else SOURCE_MANIFEST,
    )
    print(json.dumps(stats.__dict__, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
