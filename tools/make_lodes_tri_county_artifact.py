#!/usr/bin/env python3
"""Detroit tri-county LODES OD/crosswalk artifact generator (Vol II Program, Unit U2).

Reads the raw LODES OD ``.csv.gz`` files + ``us_xwalk.csv.gz`` crosswalk from the
``babylon-data`` drive (build-time-only — this script is NEVER invoked by tests or
CI; the CI-no-drive rule forbids that) and re-aggregates them, through the real,
unmodified :class:`~babylon.data.lodes_commute_matrix.LODESCommuteMatrixLoader`
filter/aggregation semantics (:meth:`_read_one_state_file`), into a synthetic-but-
faithful pair of checked-in ``.csv.gz`` files at hex resolution:

- ``src/babylon/data/reference/lodes/tri_county_hex_xwalk.csv.gz`` — one row per
  Detroit tri-county H3 res-7 cell, using the cell id itself as a synthetic LODES
  block geocode (``tabblk2020``) and the cell's own centroid as
  (``blklatdd``, ``blklondd``). H3 guarantees
  ``latlng_to_cell(cell_to_latlng(c), 7) == c``, so this round-trips losslessly
  through :meth:`LODESCommuteMatrixLoader._build_block_to_hex_map` (verified
  empirically for every cell this script emits — see :func:`_write_crosswalk`).
- ``src/babylon/data/reference/lodes/od/mi_od_main_JT00_<year>.csv.gz`` — one row
  per (origin_hex, dest_hex_or_rest_of_usa) pair with ``S000`` already summed to
  the aggregate LODES worker count for that pair — i.e. exactly the
  ``pair_counts`` / ``boundary_dest_kind`` a real block-level run against the
  full national data would compute for the tri-county study area, at a fraction
  of the row count (thousands of rows/year vs ~1.2M raw census-block rows).

Shipping the AGGREGATE matrix this way (rather than raw census-block rows) is
explicitly sanctioned by the Vol II program prompt (§3): "ship the OD matrix as
a hash-stamped deterministic artifact ... per the parquet pipeline's
conventions." It also keeps the file small enough for the ADR076 Tier-1
in-repo-CSV convention (``src/babylon/data/reference/``) — no ci-data release
upload required.

Because the synthetic files reuse ``LODESCommuteMatrixLoader`` unchanged, this
is NOT a second CSR-matrix producer (Constitution II.12: the module stays sole
producer) — it just feeds the same code a smaller, checked-in, deterministic
input.

Determinism discipline: rows sorted by primary key, explicit column set,
gzip ``compresslevel`` pinned explicitly below (Python's gzip module already
defaults to level 9, so this pin is currently a no-op — it guards against a
future stdlib default change), AND the gzip header's ``MTIME`` field pinned to
0 via :func:`_open_deterministic_gzip_text`. This does NOT mirror
``tools/make_data_artifacts.py``: that generator sidesteps this whole bug
class by writing plain, uncompressed CSV (``_write_csv`` — no ``gzip`` import
at all), so it has no header to pin. This tool's output IS gzip-compressed
(ADR076 Tier-1 convention), and ``gzip.open()``/``GzipFile`` embed the
wall-clock build timestamp in the header unless ``mtime`` is passed
explicitly — without the pin, two runs over byte-identical decompressed
content produce different sha256 digests, which defeats the "hash-stamped
deterministic artifact" contract this tool exists to satisfy (see the
program-prompt quote above).

Usage (build-time only; requires the babylon-data drive mounted)::

    uv run python tools/make_lodes_tri_county_artifact.py \\
        --lodes-root /media/user/data/babylon-data/lodes

See also: ``data-artifacts.yaml`` (``lodes_od_tri_county_hex`` /
``lodes_xwalk_tri_county_hex`` entries, added by hand — no sqlite table backs
this artifact, so it is NOT part of ``tools/make_data_artifacts.py``'s managed
``ARTIFACTS`` tuple), ``data-catalog.yaml`` (``LODES_OD_HEX`` source).
"""

from __future__ import annotations

import argparse
import csv
import gzip
import hashlib
import io
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "src"))

import h3  # noqa: E402

from babylon.data.lodes_commute_matrix import (  # noqa: E402
    LODESCommuteMatrixLoader,
    build_year_matrix,
)
from babylon.data.lodes_study_area import (  # noqa: E402
    LODES_STUDY_AREA_STATES,
    lodes_tri_county_hexes_res7,
)
from babylon.data.node_kinds import NodeKind  # noqa: E402

_GZIP_COMPRESSLEVEL = 9
_YEAR_FILE_RE = re.compile(r"^mi_od_main_JT00_(\d{4})\.csv\.gz$")


class ArtifactGenerationError(Exception):
    """A generation step failed loudly — bad input, or an output that fails
    the round-trip / internal-consistency proof."""


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    digest.update(path.read_bytes())
    return digest.hexdigest()


def _open_deterministic_gzip_text(path: Path, compresslevel: int) -> io.TextIOWrapper:
    """Open ``path`` for gzip text-mode writing with a pinned header ``MTIME``.

    ``gzip.open()`` has no ``mtime`` parameter — for a plain filename it always
    constructs :class:`gzip.GzipFile` with ``mtime=None``, which makes the
    header embed ``time.time()`` at call time. Two runs of this generator over
    byte-identical decompressed content therefore produce different sha256
    digests, which is exactly the "hash-stamped deterministic artifact"
    contract this module's module docstring commits to — and mirrors, at the
    artifact-file level, the same byte-identical-rebuild discipline ADR098
    established for the sqlite reference-DB build product. Constructing
    ``GzipFile`` directly with ``mtime=0`` pins that field; the rest — text
    wrapping via :class:`io.TextIOWrapper` — mirrors exactly what
    ``gzip.open(path, mode="wt")`` does internally.
    """
    binary = gzip.GzipFile(filename=str(path), mode="wb", compresslevel=compresslevel, mtime=0)
    return io.TextIOWrapper(binary, encoding="utf-8", newline="")


def _discover_years(lodes_root: Path) -> list[int]:
    """Every year with a Michigan ``_main_`` OD file under ``lodes_root/od/``."""
    years: list[int] = []
    for path in sorted((lodes_root / "od").glob("mi_od_main_JT00_*.csv.gz")):
        match = _YEAR_FILE_RE.match(path.name)
        if match:
            years.append(int(match.group(1)))
    return sorted(years)


def _build_mi_restricted_block_to_hex(crosswalk_path: Path) -> dict[str, str]:
    """Block -> H3 res-7 cell map, restricted to Michigan (FIPS prefix ``26``).

    Equivalent to :meth:`LODESCommuteMatrixLoader._build_block_to_hex_map` for
    this generator's purposes: LODES ``_main_`` OD files are same-state-only
    (both ``h_geocode`` and ``w_geocode`` share the requesting state's FIPS
    prefix — cross-state commutes live in the separate ``_aux_`` files, out of
    scope per spec-063 research §4), so restricting the crosswalk build to
    Michigan rows produces IDENTICAL ``block_to_hex`` resolutions for every
    block that can appear in ``mi_od_main_JT00_*.csv.gz`` — at a fraction of
    the cost of scanning the full ~8M-row national crosswalk.
    """
    out: dict[str, str] = {}
    with gzip.open(crosswalk_path, mode="rt", newline="") as fh:
        reader = csv.DictReader(fh)
        for row in reader:
            blk = row["tabblk2020"]
            if not blk.startswith("26"):
                continue
            lat_str = row.get("blklatdd", "")
            lng_str = row.get("blklondd", "")
            if not lat_str or not lng_str:
                continue
            try:
                lat = float(lat_str)
                lng = float(lng_str)
            except ValueError:
                continue
            out[blk] = h3.latlng_to_cell(lat, lng, 7)
    return out


def _aggregate_year(
    loader: LODESCommuteMatrixLoader, year: int
) -> tuple[dict[tuple[str, str], int], dict[str, NodeKind]]:
    """Run the REAL, unmodified filter/aggregation for one year.

    Delegates to :meth:`LODESCommuteMatrixLoader._read_one_state_file` — the
    exact production code path — over the real Michigan OD file. The loader's
    ``_block_to_hex`` must already be populated (see :func:`main`).
    """
    file_path = loader._resolve_state_file("26", year)  # noqa: SLF001
    if file_path is None:
        msg = f"no Michigan _main_ OD file for year={year} under {loader.lodes_root}"
        raise ArtifactGenerationError(msg)
    pair_counts: dict[tuple[str, str], int] = {}
    boundary_dest_kind: dict[str, NodeKind] = {}
    loader._read_one_state_file(  # noqa: SLF001
        file_path=file_path, pair_counts=pair_counts, boundary_dest_kind=boundary_dest_kind
    )
    return pair_counts, boundary_dest_kind


def _write_od_year(out_path: Path, pair_counts: dict[tuple[str, str], int]) -> tuple[int, str]:
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with _open_deterministic_gzip_text(out_path, _GZIP_COMPRESSLEVEL) as fh:
        writer = csv.writer(fh, lineterminator="\n")
        writer.writerow(["w_geocode", "h_geocode", "S000"])
        for (origin_hex, dest_id), s000 in sorted(pair_counts.items()):
            writer.writerow([dest_id, origin_hex, s000])
    return len(pair_counts), _sha256(out_path)


def _write_crosswalk(out_path: Path, hex_ids: set[str]) -> tuple[int, str]:
    """Write the synthetic hex-as-block crosswalk; verify the H3 round-trip
    for every emitted cell before writing (fail loud on any mismatch —
    Constitution III.8, no silently-wrong reference data)."""
    rows: list[tuple[str, float, float]] = []
    for cell in sorted(hex_ids):
        lat, lng = h3.cell_to_latlng(cell)
        back = h3.latlng_to_cell(lat, lng, 7)
        if back != cell:
            msg = f"H3 centroid round-trip failed for {cell!r}: got {back!r}"
            raise ArtifactGenerationError(msg)
        rows.append((cell, lat, lng))

    out_path.parent.mkdir(parents=True, exist_ok=True)
    with _open_deterministic_gzip_text(out_path, _GZIP_COMPRESSLEVEL) as fh:
        writer = csv.writer(fh, lineterminator="\n")
        writer.writerow(["tabblk2020", "blklatdd", "blklondd"])
        for cell, lat, lng in rows:
            writer.writerow([cell, lat, lng])
    return len(rows), _sha256(out_path)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--lodes-root",
        type=Path,
        default=Path("/media/user/data/babylon-data/lodes"),
        help="Raw LODES root (drive-mounted; read ONLY by this build-time tool).",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=Path("src/babylon/data/reference/lodes"),
        help="Checked-in output directory (repo-relative).",
    )
    parser.add_argument(
        "--years",
        type=str,
        default=None,
        help="Comma-separated years to include; default: every year with a "
        "Michigan _main_ file under --lodes-root/od/.",
    )
    args = parser.parse_args(argv)

    lodes_root: Path = args.lodes_root
    out_dir: Path = args.out_dir
    if not lodes_root.is_dir():
        msg = f"--lodes-root does not exist: {lodes_root}"
        raise ArtifactGenerationError(msg)

    years = (
        [int(y) for y in args.years.split(",") if y.strip()]
        if args.years
        else _discover_years(lodes_root)
    )
    if not years:
        msg = f"no years discovered under {lodes_root / 'od'}"
        raise ArtifactGenerationError(msg)

    study_hexes = lodes_tri_county_hexes_res7()
    print(f"[lodes-artifact] study area: {len(study_hexes)} H3 res-7 cells")

    crosswalk_path = lodes_root / "us_xwalk.csv.gz"
    print(f"[lodes-artifact] building Michigan-restricted block->hex map from {crosswalk_path} ...")
    block_to_hex = _build_mi_restricted_block_to_hex(crosswalk_path)
    print(f"[lodes-artifact] {len(block_to_hex)} Michigan blocks resolved")

    loader = LODESCommuteMatrixLoader(
        lodes_root=lodes_root,
        crosswalk_path=crosswalk_path,
        study_area_hexes=study_hexes,
        study_area_states=LODES_STUDY_AREA_STATES,
    )
    loader._block_to_hex = block_to_hex  # noqa: SLF001 — short-circuit the (correct but
    # 8M-row-national-scan-slow) production build; see _build_mi_restricted_block_to_hex.

    all_hex_ids: set[str] = set()
    per_year_report: list[tuple[int, int, str]] = []
    for year in years:
        pair_counts, boundary_dest_kind = _aggregate_year(loader, year)
        # Prove internal consistency via the REAL assembly function — any
        # malformed pair_counts/boundary_dest_kind fails loud here.
        matrix = build_year_matrix(
            pair_counts=pair_counts, boundary_dest_kind=boundary_dest_kind, year=year
        )
        all_hex_ids.update(matrix.origin_hex_to_row.keys())
        all_hex_ids.update(
            dest_id
            for dest_id, kind in zip(
                matrix.dest_node_id_by_col, matrix.dest_kind_by_col, strict=True
            )
            if kind == NodeKind.HEX
        )
        od_out = out_dir / "od" / f"mi_od_main_JT00_{year}.csv.gz"
        rows, sha = _write_od_year(od_out, pair_counts)
        per_year_report.append((year, rows, sha))
        print(f"[lodes-artifact] {year}: {rows} hex-pair rows -> {od_out} (sha256={sha})")

    xwalk_out = out_dir / "tri_county_hex_xwalk.csv.gz"
    xwalk_rows, xwalk_sha = _write_crosswalk(xwalk_out, all_hex_ids)
    print(f"[lodes-artifact] crosswalk: {xwalk_rows} cells -> {xwalk_out} (sha256={xwalk_sha})")

    print(
        "\n[lodes-artifact] data-artifacts.yaml entries (paste manually — no sqlite table backs these):"
    )
    total_rows = sum(rows for _year, rows, _sha in per_year_report)
    print(f"  lodes_od_tri_county_hex: {total_rows} total rows across {len(years)} years")
    for year, rows, sha in per_year_report:
        print(f"    {year}: rows={rows} sha256={sha}")
    print(f"  lodes_xwalk_tri_county_hex: rows={xwalk_rows} sha256={xwalk_sha}")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except ArtifactGenerationError as error:
        print(f"[lodes-artifact] ABORT: {error}", file=sys.stderr)
        sys.exit(2)
