"""Load LEHD LODES OD files into immutable sparse matrices for reference artifact generation."""

from __future__ import annotations

import csv
import gzip
import logging
from functools import lru_cache
from pathlib import Path
from typing import Any

import h3
import numpy as np
import scipy.sparse as sp  # type: ignore[import-untyped]
from pydantic import BaseModel, ConfigDict, Field, field_validator, model_validator

from babylon.data.node_kinds import NodeKind

logger = logging.getLogger(__name__)


# ─────────────────────────────────────────────────────────────────────────────
# LODESYearMatrix — frozen wrapper around the loaded CSR matrix (T015).
# ─────────────────────────────────────────────────────────────────────────────


class LODESYearMatrix(BaseModel):
    """Year-scoped LEHD LODES OD matrix in scipy.sparse CSR form.

    Frozen + arbitrary_types_allowed because :class:`scipy.sparse.csr_matrix`
    is a non-Pydantic type. The underlying matrix is treated as immutable
    once constructed (Constitution II.6 / FR-006).
    """

    model_config = ConfigDict(frozen=True, arbitrary_types_allowed=True)

    year: int = Field(ge=1900, le=2099, description="The simulated year this matrix represents.")
    matrix: Any = Field(
        description="scipy.sparse.csr_matrix shape (n_origins, n_destinations); dtype=float64."
    )
    origin_hex_to_row: dict[str, int] = Field(
        description="H3 res-7 origin cell → matrix row index."
    )
    dest_to_col: dict[str, int] = Field(
        description="Destination ID (hex or boundary bucket) → matrix col index."
    )
    dest_kind_by_col: tuple[NodeKind, ...] = Field(
        description="One NodeKind entry per matrix column."
    )
    dest_node_id_by_col: tuple[str, ...] = Field(
        description="One destination ID string per matrix column."
    )
    row_sums: Any = Field(description="Cached numpy.ndarray row-sum vector (length n_origins).")

    @field_validator("matrix")
    @classmethod
    def _matrix_must_be_csr(cls, value: Any) -> Any:  # noqa: ANN401 — scipy isn't Pydantic-typed
        if not sp.issparse(value):
            raise TypeError(f"matrix must be a scipy.sparse matrix; got {type(value).__name__}")
        if value.format != "csr":
            raise ValueError(
                f"matrix must be CSR format per Constitution II.12 GATE-4; got {value.format}"
            )
        if value.dtype != np.float64:
            raise ValueError(f"matrix dtype must be float64; got {value.dtype}")
        if (value.data < 0).any():
            raise ValueError(
                "matrix contains negative entries; LODES worker counts are non-negative"
            )
        return value

    @field_validator("row_sums")
    @classmethod
    def _row_sums_must_be_ndarray(cls, value: Any) -> Any:  # noqa: ANN401
        if not isinstance(value, np.ndarray):
            raise TypeError(f"row_sums must be numpy.ndarray; got {type(value).__name__}")
        if value.ndim != 1:
            raise ValueError(f"row_sums must be 1-D; got shape {value.shape}")
        return value

    @model_validator(mode="after")
    def _check_internal_consistency(self) -> LODESYearMatrix:
        n_rows, n_cols = self.matrix.shape
        if len(self.origin_hex_to_row) != n_rows:
            raise ValueError(
                f"origin_hex_to_row size {len(self.origin_hex_to_row)} != matrix rows {n_rows}"
            )
        if len(self.dest_to_col) != n_cols:
            raise ValueError(f"dest_to_col size {len(self.dest_to_col)} != matrix cols {n_cols}")
        if len(self.dest_kind_by_col) != n_cols:
            raise ValueError(
                f"dest_kind_by_col length {len(self.dest_kind_by_col)} != matrix cols {n_cols}"
            )
        if len(self.dest_node_id_by_col) != n_cols:
            raise ValueError(
                f"dest_node_id_by_col length {len(self.dest_node_id_by_col)} != matrix cols {n_cols}"
            )
        if self.row_sums.shape != (n_rows,):
            raise ValueError(f"row_sums shape {self.row_sums.shape} != ({n_rows},)")
        # Spot-check row_sums consistency without paying for full recomputation.
        computed = np.asarray(self.matrix.sum(axis=1)).ravel()
        if not np.allclose(computed, self.row_sums):
            raise ValueError("row_sums do not match matrix.sum(axis=1) — corruption detected")
        return self

    def dest_kind_breakdown(self) -> dict[str, int]:
        """Count destination columns by NodeKind. Useful for tests and walkthroughs."""
        out: dict[str, int] = {}
        for kind in self.dest_kind_by_col:
            out[kind.value] = out.get(kind.value, 0) + 1
        return out

    def dest_kind_breakdown_str(self) -> str:
        """Human-readable form of :meth:`dest_kind_breakdown` for log output."""
        items = sorted(self.dest_kind_breakdown().items())
        return ", ".join(f"{k}={v}" for k, v in items)


def build_year_matrix(
    *,
    pair_counts: dict[tuple[str, str], int],
    boundary_dest_kind: dict[str, NodeKind],
    year: int,
) -> LODESYearMatrix:
    """Assemble a :class:`LODESYearMatrix` from aggregated pair counts.

    Module-level so producers without a :class:`LODESCommuteMatrixLoader`
    instance can build a matrix from the same deterministic sorted-pair CSR
    assembly. Spec-063 T042 reuses this for the in-memory border-commute
    merge (:meth:`babylon.domain.economics.border_commute_synthesis.BorderCommuteSynthesisLoader.merge_into_year_matrix`).
    :meth:`LODESCommuteMatrixLoader._build_csr_matrix` delegates here so the
    on-disk parse, Postgres read-back, and merge paths stay byte-identical
    (Constitution III.7 determinism).

    Args:
        pair_counts: ``(origin_hex, dest_node_id) -> S000 worker count``.
        boundary_dest_kind: ``dest_node_id -> NodeKind`` (HEX for in-area
            destinations, EXTERNAL for boundary buckets). Missing entries
            default to :attr:`NodeKind.HEX`.
        year: The simulated year this matrix represents.

    Returns:
        A frozen :class:`LODESYearMatrix` whose CSR entries, column ordering,
        and cached row-sums are a pure function of the inputs.
    """
    if not pair_counts:
        # Empty matrix is valid — represents a year with no in-area commute (degenerate test case)
        data = np.zeros(0, dtype=np.float64)
        indices = np.zeros(0, dtype=np.int32)
        indptr = np.zeros(1, dtype=np.int32)
        matrix = sp.csr_matrix((data, indices, indptr), shape=(0, 0), dtype=np.float64)
        row_sums = np.zeros(0, dtype=np.float64)
        return LODESYearMatrix(
            year=year,
            matrix=matrix,
            origin_hex_to_row={},
            dest_to_col={},
            dest_kind_by_col=(),
            dest_node_id_by_col=(),
            row_sums=row_sums,
        )

    origins = sorted({h for (h, _) in pair_counts})
    dests = sorted({d for (_, d) in pair_counts})
    origin_hex_to_row = {h: i for i, h in enumerate(origins)}
    dest_to_col = {d: i for i, d in enumerate(dests)}
    dest_kind_by_col = tuple(boundary_dest_kind.get(d, NodeKind.HEX) for d in dests)
    dest_node_id_by_col = tuple(dests)

    rows = np.empty(len(pair_counts), dtype=np.int32)
    cols = np.empty(len(pair_counts), dtype=np.int32)
    vals = np.empty(len(pair_counts), dtype=np.float64)
    for idx, ((h, d), v) in enumerate(sorted(pair_counts.items())):
        rows[idx] = origin_hex_to_row[h]
        cols[idx] = dest_to_col[d]
        vals[idx] = float(v)
    coo = sp.coo_matrix((vals, (rows, cols)), shape=(len(origins), len(dests)), dtype=np.float64)
    matrix = coo.tocsr()
    row_sums = np.asarray(matrix.sum(axis=1)).ravel().astype(np.float64)

    return LODESYearMatrix(
        year=year,
        matrix=matrix,
        origin_hex_to_row=origin_hex_to_row,
        dest_to_col=dest_to_col,
        dest_kind_by_col=dest_kind_by_col,
        dest_node_id_by_col=dest_node_id_by_col,
        row_sums=row_sums,
    )


# ─────────────────────────────────────────────────────────────────────────────
# LODESCommuteMatrixLoader — disk + Postgres I/O for the year-scoped matrix (T016/T017).
# ─────────────────────────────────────────────────────────────────────────────


_LODES_FILE_PATTERN = "{state}_od_main_JT00_{year}.csv.gz"


class LODESCommuteMatrixLoader:
    """Read on-disk LODES OD CSVs + serve year-scoped CSR matrices.

    Loader is constructed once per session. ``load_year`` is idempotent within
    a process (cached). ``load_year_from_postgres`` rebuilds the in-memory CSR
    from Rust-owned reference rows for read-only analysis paths.

    Per Constitution II.13 GATE-5: this is the *deterministic min-cost flow*
    component. Slime-mold conductivity routing is implemented in spec 064 as
    a separate overlay; do not add conductivity logic here.
    """

    def __init__(
        self,
        *,
        lodes_root: Path,
        crosswalk_path: Path,
        study_area_hexes: frozenset[str],
        study_area_states: frozenset[str],
    ) -> None:
        if not lodes_root.exists():
            raise FileNotFoundError(f"LODES root does not exist: {lodes_root}")
        if not crosswalk_path.exists():
            raise FileNotFoundError(f"LODES crosswalk does not exist: {crosswalk_path}")
        if not study_area_hexes:
            raise ValueError(
                "study_area_hexes MUST be non-empty (loader needs a study area to prune)"
            )
        if not study_area_states:
            raise ValueError("study_area_states MUST be non-empty")
        for code in study_area_states:
            if not (len(code) == 2 and code.isdigit()):
                raise ValueError(f"study_area_states entry {code!r} is not a 2-digit FIPS code")

        self.lodes_root = lodes_root
        self.crosswalk_path = crosswalk_path
        self.study_area_hexes = study_area_hexes
        self.study_area_states = study_area_states
        self._year_cache: dict[int, LODESYearMatrix] = {}
        # Block-code → H3 res-7 cell map. Built lazily on first load_year() call.
        self._block_to_hex: dict[str, str] | None = None

    # ── Disk loading ────────────────────────────────────────────────────────

    def load_year(self, year: int) -> LODESYearMatrix:
        """Build (or return cached) :class:`LODESYearMatrix` for ``year``.

        Honors FR-004: if ``year`` is outside :meth:`available_years`, the
        loader uses the nearest available year and emits one log entry per
        substitution per process. Still returns a matrix tagged with the
        *originally requested* year for caller-side bookkeeping; the
        ``od_year_used`` field on :class:`CirculationStepResult` carries the
        actually-consumed year for forensic determinism (FR-005).
        """
        clamped = self.clamp_to_available(year)
        if clamped != year:
            logger.warning(
                "LODES year %d not available; clamping to %d (FR-004 nearest-year clamp)",
                year,
                clamped,
            )
        if clamped in self._year_cache:
            return self._year_cache[clamped]

        if self._block_to_hex is None:
            self._block_to_hex = self._build_block_to_hex_map()

        # Aggregate (origin_hex, dest_id) → worker count across all in-state files.
        pair_counts: dict[tuple[str, str], int] = {}
        boundary_dest_kind: dict[str, NodeKind] = {}
        for state in sorted(self.study_area_states):
            # LODES uses USPS state abbreviations (mi, ny, ca) in filenames, NOT FIPS codes.
            # Resolve FIPS → USPS-suffix path via the per-state helper.
            file_path = self._resolve_state_file(state, clamped)
            if file_path is None:
                logger.warning("No LODES file for state=%s year=%d; skipping", state, clamped)
                continue
            self._read_one_state_file(
                file_path=file_path,
                pair_counts=pair_counts,
                boundary_dest_kind=boundary_dest_kind,
            )

        matrix = self._build_csr_matrix(
            pair_counts=pair_counts,
            boundary_dest_kind=boundary_dest_kind,
            year=clamped,
        )
        self._year_cache[clamped] = matrix
        return matrix

    def available_years(self) -> tuple[int, ...]:
        """Return sorted tuple of years for which any in-state ``_main_`` file exists."""
        years: set[int] = set()
        for state in self.study_area_states:
            usps = self._fips_to_usps(state)
            if usps is None:
                continue
            for path in (self.lodes_root / "od").glob(f"{usps}_od_main_JT00_*.csv.gz"):
                # Filename: <usps>_od_main_JT00_<year>.csv.gz
                stem = path.name.removesuffix(".csv.gz")
                year_str = stem.rsplit("_", 1)[-1]
                if year_str.isdigit():
                    years.add(int(year_str))
        return tuple(sorted(years))

    def clamp_to_available(self, target_year: int) -> int:
        """Return the available year nearest to ``target_year`` per FR-004.

        If ``target_year`` is in the available set, returns it unchanged.
        Otherwise returns whichever available year minimizes ``|y - target_year|``;
        ties broken by preferring the LATER year.
        """
        years = self.available_years()
        if not years:
            raise FileNotFoundError(
                f"No LODES files found for states {sorted(self.study_area_states)} in {self.lodes_root / 'od'}"
            )
        if target_year in years:
            return target_year
        # Choose nearest; ties → later year (better than older data for forward extrapolation)
        return min(years, key=lambda y: (abs(y - target_year), -y))

    # ── Internal: file parsing + matrix assembly ────────────────────────────

    def _resolve_state_file(self, fips_state: str, year: int) -> Path | None:
        """Resolve the LODES ``_main_`` file for a FIPS state code + year."""
        usps = self._fips_to_usps(fips_state)
        if usps is None:
            return None
        path = self.lodes_root / "od" / f"{usps}_od_main_JT00_{year}.csv.gz"
        return path if path.exists() else None

    @staticmethod
    @lru_cache(maxsize=64)
    def _fips_to_usps(fips: str) -> str | None:
        """Map FIPS 2-digit state code → lowercase USPS abbreviation for filename use.

        Hardcoded for the 50 states + DC + 5 territories that LODES publishes.
        """
        table = {
            "01": "al",
            "02": "ak",
            "04": "az",
            "05": "ar",
            "06": "ca",
            "08": "co",
            "09": "ct",
            "10": "de",
            "11": "dc",
            "12": "fl",
            "13": "ga",
            "15": "hi",
            "16": "id",
            "17": "il",
            "18": "in",
            "19": "ia",
            "20": "ks",
            "21": "ky",
            "22": "la",
            "23": "me",
            "24": "md",
            "25": "ma",
            "26": "mi",
            "27": "mn",
            "28": "ms",
            "29": "mo",
            "30": "mt",
            "31": "ne",
            "32": "nv",
            "33": "nh",
            "34": "nj",
            "35": "nm",
            "36": "ny",
            "37": "nc",
            "38": "nd",
            "39": "oh",
            "40": "ok",
            "41": "or",
            "42": "pa",
            "44": "ri",
            "45": "sc",
            "46": "sd",
            "47": "tn",
            "48": "tx",
            "49": "ut",
            "50": "vt",
            "51": "va",
            "53": "wa",
            "54": "wv",
            "55": "wi",
            "56": "wy",
        }
        return table.get(fips)

    def _read_one_state_file(
        self,
        *,
        file_path: Path,
        pair_counts: dict[tuple[str, str], int],
        boundary_dest_kind: dict[str, NodeKind],
    ) -> None:
        """Stream-read one LODES `_main_` file, prune to study-area, aggregate by hex pair."""
        block_to_hex = self._block_to_hex
        if block_to_hex is None:
            raise RuntimeError("internal: _block_to_hex must be built before _read_one_state_file")

        in_area = self.study_area_hexes
        with gzip.open(file_path, mode="rt", newline="") as fh:
            reader = csv.DictReader(fh)
            for row in reader:
                w_geo = row["w_geocode"]
                h_geo = row["h_geocode"]
                s000 = int(row["S000"])
                if s000 == 0:
                    continue
                # Map both block codes to hexes; skip pairs touching no in-area cell.
                home_hex = block_to_hex.get(h_geo)
                work_hex = block_to_hex.get(w_geo)
                home_in = home_hex is not None and home_hex in in_area
                work_in = work_hex is not None and work_hex in in_area
                if not (home_in or work_in):
                    continue  # Neither end touches the study area — drop per FR-007
                # Build origin / dest. Origin must be in-area (or we skip — out-of-area origins
                # are tracked separately by the future "_aux_" file path; not in spec 063 scope).
                if not home_in:
                    continue
                assert home_hex is not None  # noqa: S101 — mypy narrowing; unreachable per the preceding continue guard
                if work_in:
                    assert work_hex is not None  # noqa: S101 — mypy narrowing; unreachable per the preceding continue guard
                    dest_id = work_hex
                    boundary_dest_kind[dest_id] = NodeKind.HEX
                else:
                    # Out-of-area destination — bucket as rest_of_usa for now.
                    # (Detroit-Windsor classification happens at emission time, not here.)
                    dest_id = "rest_of_usa"
                    boundary_dest_kind[dest_id] = NodeKind.EXTERNAL
                key = (home_hex, dest_id)
                pair_counts[key] = pair_counts.get(key, 0) + s000

    def _build_csr_matrix(
        self,
        *,
        pair_counts: dict[tuple[str, str], int],
        boundary_dest_kind: dict[str, NodeKind],
        year: int,
    ) -> LODESYearMatrix:
        """Assemble a :class:`LODESYearMatrix` from aggregated pair counts.

        Thin instance-method delegate to the module-level
        :func:`build_year_matrix` (extracted for reuse by the spec-063 T042
        border-commute merge). Uses no instance state.
        """
        return build_year_matrix(
            pair_counts=pair_counts, boundary_dest_kind=boundary_dest_kind, year=year
        )

    def _build_block_to_hex_map(self) -> dict[str, str]:
        """One-time-per-process build of the LODES block → H3 res-7 cell map.

        Reads ``us_xwalk.csv.gz`` (143 MB), uses ``blklatdd``/``blklondd``
        columns to compute ``h3.latlng_to_cell(lat, lng, 7)`` per block.
        Result is cached on the loader instance to amortize across years.
        """
        logger.info("Building LODES block→hex map from %s ...", self.crosswalk_path)
        out: dict[str, str] = {}
        with gzip.open(self.crosswalk_path, mode="rt", newline="") as fh:
            reader = csv.DictReader(fh)
            for row in reader:
                lat_str = row.get("blklatdd", "")
                lng_str = row.get("blklondd", "")
                if not lat_str or not lng_str:
                    continue
                try:
                    lat = float(lat_str)
                    lng = float(lng_str)
                except ValueError:
                    continue
                cell = h3.latlng_to_cell(lat, lng, 7)
                out[row["tabblk2020"]] = cell
        logger.info("LODES block→hex map built: %d entries", len(out))
        return out


__all__ = [
    "LODESCommuteMatrixLoader",
    "LODESYearMatrix",
    "build_year_matrix",
]
