#!/usr/bin/env python3
"""FAF freight bloc-tons artifact generator (Program 26, Unit U3).

Reads the raw FAF5 State-level O-D freight flow CSV
(``data/freight/faf/FAF5.7.1_State_2018-2024.csv``, symlinked to the
``babylon-data`` drive — 1.1M rows, streamed row-by-row, never loaded whole)
and aggregates annual tons per FAF foreign region (``fr_orig``/``fr_dest``,
codes 801-808), then maps those 8 FAF regions onto the engine's 8
:data:`~babylon.persistence.postgres_initialization.INTERNATIONAL_NODES` via
an explicit, disclosed, injective-per-region table (:data:`FAF_ZONE_TO_NODE`
below), producing a small checked-in artifact:

- ``src/babylon/data/reference/faf_bloc_trade_tons.csv.gz`` — columns
  ``node_id, year, tons`` (thousand tons, FAF's native unit — see
  ``FAF5_metadata.xlsx``'s "Data Dictionary" sheet: ``tons`` = "Total weight
  of commodities shipped (unit: Thousand Tons)"), one row per (mapped node,
  year) for years 2018-2024 (FAF5.7.1's covered span).

FAF zone -> engine node mapping (source: ``FAF5_metadata.xlsx``'s "FAF Zone
(Foreign)" sheet for the 8 zone descriptions; zone 806's composition
independently confirmed via the FHWA/BTS FAF5 foreign-country-list — see
module-level ``FAF_ZONE_TO_NODE`` comment below for the full disclosure):

===== ========================= ================= ======================
Zone  FAF description           -> Node            Fidelity note
===== ========================= ================= ======================
801   Canada                    canada             exact match
802   Mexico                    latin_america      no dedicated NAFTA/
                                                    Mexico node; nearest
                                                    fit is latin_america
                                                    (Census-style "Latin
                                                    America" groupings
                                                    include Mexico)
803   Rest of Americas          latin_america       direct fit (non-US/
                                                    Canada/Mexico Western
                                                    Hemisphere)
804   Europe                    eu                 containing-region:
                                                    FAF's "Europe" is
                                                    broader than the EU
                                                    (UK, Switzerland,
                                                    Norway, and by
                                                    elimination Russia,
                                                    since no separate
                                                    Russia/Eurasia zone
                                                    exists) — same
                                                    imprecision class as
                                                    the existing
                                                    ``_NODE_TO_BLOC``
                                                    ``"eu": 1`` mapping
805   Africa                    sub_saharan_africa containing-region
                                                    (includes North
                                                    Africa) — mirrors
                                                    ``_NODE_TO_BLOC``'s
                                                    ``"sub_saharan_africa"``
                                                    disclosure exactly
806   SW & Central Asia         EXCLUDED           confirmed (FHWA/BTS
                                                    FAF5 foreign-zone
                                                    country list) to mix
                                                    India + the Middle
                                                    East (Iran, Iraq,
                                                    Saudi Arabia, Israel,
                                                    ...) + Central
                                                    Asia/Caucasus
                                                    (Kazakhstan,
                                                    Uzbekistan, Georgia,
                                                    Armenia, Azerbaijan)
                                                    + Turkey. No single
                                                    INTERNATIONAL_NODE
                                                    honestly captures
                                                    this — the exact
                                                    "region mixing India
                                                    with the Middle East"
                                                    hazard this module's
                                                    charter names.
                                                    Excluded rather than
                                                    fabricated (III.8).
                                                    ``india`` and
                                                    ``russia_csi`` (its
                                                    Central-Asia/Caucasus
                                                    component would have
                                                    landed here) get NO
                                                    FAF-tons coverage —
                                                    ``bilateral_trade_tons``
                                                    stays 0.0 for those
                                                    two nodes even after
                                                    this artifact lands.
807   Eastern Asia              china              dominant Asian trade
                                                    partner — mirrors
                                                    ``_NODE_TO_BLOC``'s
                                                    ``"china": 12``
                                                    disclosure exactly
808   SE Asia & Oceania         southeast_asia     containing-region
                                                    (adds Oceania) —
                                                    mirrors
                                                    ``_NODE_TO_BLOC``'s
                                                    ``"southeast_asia"``
                                                    disclosure exactly
===== ========================= ================= ======================

Determinism discipline mirrors ``tools/make_lodes_tri_county_artifact.py``
(ADR121 hand-maintained-artifact pattern): rows sorted by primary key
(``node_id``, ``year``), explicit column set, gzip header ``MTIME`` pinned to
0 so two runs over byte-identical decompressed content produce identical
sha256 digests.

This tool does NOT write ``data-artifacts.yaml`` itself — like
``make_lodes_tri_county_artifact.py`` and ``make_election_lab_artifact.py``,
it prints the rows/sha256 for hand-entry into the manifest (no backing
sqlite reference-DB table; ``make_data_artifacts.py``'s ``ARTIFACTS`` tuple
never names it — see the ``EXCEPTION`` note at the top of
``data-artifacts.yaml``). ``tests/unit/tools/test_faf_artifact_manifest_entry.py``
is the tripwire.

Usage (build-time only; requires the FAF CSV under ``data/freight/faf/``,
symlinked to the ``babylon-data`` drive)::

    uv run python tools/make_faf_bloc_tons_artifact.py \\
        --faf-csv data/freight/faf/FAF5.7.1_State_2018-2024.csv
"""

from __future__ import annotations

import argparse
import csv
import gzip
import hashlib
import io
import sys
from pathlib import Path

_GZIP_COMPRESSLEVEL = 9

#: FAF5.7.1's covered annual span (the ``tons_YYYY`` column suffixes present
#: in ``FAF5.7.1_State_2018-2024.csv``).
FAF_YEARS: tuple[int, ...] = (2018, 2019, 2020, 2021, 2022, 2023, 2024)

#: FAF foreign-zone numeric code -> zone description (from
#: ``FAF5_metadata.xlsx``'s "FAF Zone (Foreign)" sheet — the empirical,
#: decoded set of ``fr_orig``/``fr_dest`` values found in the CSV).
FAF_ZONE_DESCRIPTION: dict[int, str] = {
    801: "Canada",
    802: "Mexico",
    803: "Rest of Americas",
    804: "Europe",
    805: "Africa",
    806: "SW & Central Asia",
    807: "Eastern Asia",
    808: "SE Asia & Oceania",
}

#: FAF zone -> engine node, injective per zone (a zone maps to at most one
#: node; a node MAY receive more than one zone, e.g. ``latin_america`` gets
#: both Mexico and Rest of Americas). Zone 806 is deliberately ABSENT — see
#: the module docstring's disclosure table. Every
#: :data:`~babylon.persistence.postgres_initialization.INTERNATIONAL_NODES`
#: entry either appears as a value here or is accounted for as a disclosed
#: absence (india, russia_csi) — enforced by
#: ``tests/unit/tools/test_faf_bloc_mapping.py``.
FAF_ZONE_TO_NODE: dict[int, str] = {
    801: "canada",
    802: "latin_america",
    803: "latin_america",
    804: "eu",
    805: "sub_saharan_africa",
    807: "china",
    808: "southeast_asia",
}

#: Nodes with NO FAF zone assignment (folded into the excluded zone 806, or
#: otherwise absent from the 8-zone FAF taxonomy) — disclosed, not silently
#: dropped. ``bilateral_trade_tons`` remains 0.0 for these two nodes.
FAF_UNCOVERED_NODES: tuple[str, ...] = ("india", "russia_csi")


class ArtifactGenerationError(Exception):
    """A generation step failed loudly — bad input or a malformed CSV."""


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    digest.update(path.read_bytes())
    return digest.hexdigest()


def _open_deterministic_gzip_text(path: Path, compresslevel: int) -> io.TextIOWrapper:
    """Open ``path`` for gzip text-mode writing with a pinned header ``MTIME``.

    Mirrors ``tools/make_lodes_tri_county_artifact.py``'s
    ``_open_deterministic_gzip_text`` exactly — see that module's docstring
    for why the pin is required for a byte-identical hash contract.
    """
    binary = gzip.GzipFile(filename=str(path), mode="wb", compresslevel=compresslevel, mtime=0)
    return io.TextIOWrapper(binary, encoding="utf-8", newline="")


def aggregate_faf_tons(faf_csv: Path) -> dict[tuple[int, int], float]:
    """Stream ``faf_csv`` and sum ``tons_YYYY`` per (FAF zone, year).

    A row contributes to its ``fr_orig`` zone (import leg) when ``fr_orig``
    is non-empty, and to its ``fr_dest`` zone (export leg) when ``fr_dest``
    is non-empty (domestic rows, ``trade_type=1``, have both empty and
    contribute nothing) — verified empirically against ``trade_type``:
    ``trade_type=2`` (import) always has exactly ``fr_orig`` populated,
    ``trade_type=3`` (export) always has exactly ``fr_dest`` populated.

    :returns: ``{(zone_code, year): total_tons_thousands}`` — the FAF zone's
        total (import + export) annual tons, summed over every commodity/
        mode/distance-band row for that zone/year.
    :raises ArtifactGenerationError: If ``faf_csv`` has no header or is empty.
    """
    agg: dict[tuple[int, int], float] = {}
    with faf_csv.open(newline="") as fh:
        reader = csv.DictReader(fh)
        if reader.fieldnames is None:
            msg = f"no header row in {faf_csv}"
            raise ArtifactGenerationError(msg)
        row_count = 0
        for row in reader:
            row_count += 1
            fr_orig = row.get("fr_orig", "")
            fr_dest = row.get("fr_dest", "")
            for year in FAF_YEARS:
                raw = row.get(f"tons_{year}")
                if not raw:
                    continue
                tons = float(raw)
                if fr_orig:
                    key = (int(fr_orig), year)
                    agg[key] = agg.get(key, 0.0) + tons
                if fr_dest:
                    key = (int(fr_dest), year)
                    agg[key] = agg.get(key, 0.0) + tons
        if row_count == 0:
            msg = f"no data rows in {faf_csv}"
            raise ArtifactGenerationError(msg)
    return agg


def map_zone_tons_to_nodes(
    zone_year_tons: dict[tuple[int, int], float],
) -> dict[tuple[str, int], float]:
    """Apply :data:`FAF_ZONE_TO_NODE` to fold zone-level tons onto engine nodes.

    Multiple zones may fold onto the same node (``latin_america`` <- 802 +
    803); their tons sum. Zone 806 (and any other zone absent from
    :data:`FAF_ZONE_TO_NODE`) contributes nothing — disclosed exclusion, not
    a silent drop (see the module docstring).
    """
    out: dict[tuple[str, int], float] = {}
    for (zone, year), tons in zone_year_tons.items():
        node_id = FAF_ZONE_TO_NODE.get(zone)
        if node_id is None:
            continue
        key = (node_id, year)
        out[key] = out.get(key, 0.0) + tons
    return out


def _write_artifact(
    out_path: Path, node_year_tons: dict[tuple[str, int], float]
) -> tuple[int, str]:
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with _open_deterministic_gzip_text(out_path, _GZIP_COMPRESSLEVEL) as fh:
        writer = csv.writer(fh, lineterminator="\n")
        writer.writerow(["node_id", "year", "tons"])
        for node_id, year in sorted(node_year_tons):
            writer.writerow([node_id, year, f"{node_year_tons[(node_id, year)]:.6f}"])
    return len(node_year_tons), _sha256(out_path)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--faf-csv",
        type=Path,
        default=Path("data/freight/faf/FAF5.7.1_State_2018-2024.csv"),
        help="Raw FAF5 State-level O-D CSV (repo-relative; symlinked to the drive).",
    )
    parser.add_argument(
        "--out",
        type=Path,
        default=Path("src/babylon/data/reference/faf_bloc_trade_tons.csv.gz"),
        help="Checked-in output path (repo-relative).",
    )
    args = parser.parse_args(argv)

    faf_csv: Path = args.faf_csv
    if not faf_csv.is_file():
        msg = f"--faf-csv does not exist: {faf_csv}"
        raise ArtifactGenerationError(msg)

    print(f"[faf-artifact] aggregating {faf_csv} ...")
    zone_year_tons = aggregate_faf_tons(faf_csv)
    print(f"[faf-artifact] {len(zone_year_tons)} (zone, year) cells aggregated")

    node_year_tons = map_zone_tons_to_nodes(zone_year_tons)
    rows, sha = _write_artifact(args.out, node_year_tons)
    print(f"[faf-artifact] {rows} rows -> {args.out} (sha256={sha})")
    print(
        "\n[faf-artifact] data-artifacts.yaml entry (paste manually — no sqlite table backs this):"
    )
    print("  faf_bloc_trade_tons:")
    print(f"    rows: {rows}")
    print(f"    sha256: {sha}")
    print(f"    home: {args.out.as_posix()}")
    covered = sorted({node for node, _year in node_year_tons})
    print(f"[faf-artifact] nodes covered: {covered}")
    print(f"[faf-artifact] nodes with NO FAF coverage (disclosed): {list(FAF_UNCOVERED_NODES)}")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except ArtifactGenerationError as error:
        print(f"[faf-artifact] ABORT: {error}", file=sys.stderr)
        sys.exit(2)
