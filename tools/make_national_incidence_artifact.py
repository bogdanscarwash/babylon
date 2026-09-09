#!/usr/bin/env python3
"""National-incidence artifact generator — sha-pinned source access, the
full derivation pipeline, and A2/A3 emission (#334 Phase 0, T2-T4).

Plan: ``docs/superpowers/plans/2026-08-17-334-incidence-artifact.md`` §3 (the
derivation), §2 (A2/A3 shapes). T2 implemented plan §3's steps 1-2 (universe
resolution + filtered cell pull); T3a landed the four arithmetic-law guards
(:func:`ratio_of_sums` G1, :func:`classify_zero_denominator` G2,
:func:`assert_t_pole_exactness` G6, :func:`overlap_upper_bound` G8); T3b
landed the three absence/small-count guards (:func:`damp` +
:func:`compute_damped_sigma` G3, :func:`classify_suppression` +
:func:`classify_absence` G4, :func:`reconcile_absence_counts` +
:func:`assert_no_pine_ridge_imputation` G5). **T4 wires every guard into the
full per-county measure pipeline** (steps 3-9) and emits A2
(:func:`build_county_pole_rows` + :func:`write_county_pole_artifact`) and A3
(:func:`build_reproduction_floor_rows` + :func:`write_reproduction_floor_artifact`)
deterministically. G7 already shipped with T1 (``tools/make_fips_vintage_crosswalk.py``).
Registration (``data-artifacts.yaml``/``data-catalog.yaml`` entries) is T5's job —
this module only prints the manifest blocks for hand-entry.

ADR098 circularity, resolved the same way T1 states it: these are
**second-order products** derived from **registered parquet sources**
(``dist/data-artifacts/*.parquet``), never from the sqlite build product
directly. **There is no ``--from-sqlite`` derivation path.** The sqlite file
(``data/sqlite/marxist-data-3NF.sqlite``) is touched in exactly two places,
neither of which is "derive the poverty measures from it":

1. ``--export-source`` mode (:func:`export_source_tables`) — PRODUCES the
   registered parquet sources by calling
   ``make_data_artifacts.export_table_parquet``, the ONLY sanctioned
   parquet-writing path (``tools/make_data_artifacts.py:389-409``). This is
   how a fresh box materializes ``dist/data-artifacts/fact_census_poverty.
   parquet`` — ``dist/`` is gitignored, so the parquet is absent until this
   runs. Hard-gated on the pinned SQLite runtime (mirrors
   ``tools/build_reference_db.py``'s identical gate) — an off-pin export
   risks producing bytes that don't match the registered
   ``data-artifacts.yaml`` sha256 pins, which is exactly the kind of silent
   drift step 2 below exists to catch, so it's caught earlier and louder
   here instead. Run inside ``mise exec -- uv run --frozen python ...``
   (``docs/how-to/reference-data-pipeline.rst``).
2. The ``scopes`` universe (:func:`_scopes_universe`) reuses
   ``babylon.data.reference_scope._load_national_fips`` directly —
   the same call ``tools/make_fips_vintage_crosswalk.py`` (T1) already
   makes. This is universe-**membership** enumeration against
   ``dim_county`` (which FIPS codes exist), never a ``fact_census_poverty``
   **measure** read. It is not the circularity the ``--from-sqlite``
   prohibition closes off.

Every actual **derivation** read — the filtered poverty cells
(:func:`read_filtered_poverty_cells`) and the ``unrestricted`` universe
(:func:`_unrestricted_universe`) — reads ONLY the sha-pinned parquet sources,
verified by :func:`verify_source_provenance` BEFORE any of those reads
happen. Mismatch or a missing source raises :class:`SourceProvenanceError`,
loud, no fallback.

**Pinned filters** (OQ11: pinned 2019 vintage, never re-cut — the extractor
that could re-cut a different vintage was deleted,
``data-catalog.yaml:1096``): ``time_id=23`` (2019), ``category_id in {1, 2}``
(1 = ``B17001_001`` "Total" = the universe *u*; 2 = ``B17001_002``
"below poverty" = *b*), ``race_id in {1, ..., 10}`` — **widened from the
plan's literal ``{1, 2, 3, 4, 9, 10}`` by a controller ruling at T4** (the
plan's own guard register self-contradicts: G6 needs the seven
mutually-exclusive ``A..G`` parts (race_id 2-8) to check ``T == Σ(A..G)``,
which the narrower filter never pulls). The single pyarrow pull now carries
all ten race_id values — T, A, B, C, D, E, F, G, H, I — so G6's exactness
check runs on the REAL pulled cells; the pole derivation (T4) then selects
its four-pole subset (B/C/I/H) downstream, never re-querying. Race codes:
1=T Total, 2=A White alone, 3=B Black, 4=C AIAN/Indigenous, 5=D Asian,
6=E NHPI, 7=F Some other race, 8=G Two or more races, 9=H
White-non-Hispanic/settler reference, 10=I Hispanic/Chicano — **note the
pole-letter trap**: the charter's B/C/I nation-pole letters are NOT these
census race codes; Chicano resolves to census ``I`` (race_id 10), Indigenous
to census ``C`` (race_id 4). Plan §0.).

**A1 applied** (step 1's other half, plan §3): the checked-in crosswalk CSV
(``src/babylon/data/reference/national/county_fips_vintage_crosswalk.csv``,
T1) is **the authority** — loaded directly by :func:`load_crosswalk`, never
re-derived from ``tools/make_fips_vintage_crosswalk.py``'s in-module
``CROSSWALK_ROWS`` constant. :func:`resolve_query_fips` is the concrete
application: a ``recoverable=true`` row substitutes its ``fips_acs2019``
target (Bedford ``51515`` -> ``51019``) as the FIPS to query
``fact_census_poverty`` under; every other engine-universe county —
including ``DECLARED_HOLE`` rows (``46102``, ``02158``) and the
non-recoverable ``SPLIT_UNRESOLVED``/``REORGANIZATION_UNRESOLVED`` rows
(``02063``, ``02066``, the 9 CT planning regions) — queries its own native
FIPS, which for those rows simply returns no data. **T1's report is binding
here**: 02063/02066/02158 must NOT be treated as crosswalk-recovered.
Classifying *why* a county has no data (``ROW_ABSENT`` vs ``DECLARED_HOLE``
vs ``SUPPRESSED``) is G4/T3b's job, not this resolver's — T2 computes
nothing.

**Universe-variant naming, a verified deviation from the plan's literal
strings:** plan §2 A3 names the three ``universe_variant`` enum values
``artifact_3153``, ``scopes_3140``, ``unrestricted_3218``. T1 already
measured the scopes (resolver) universe at **3,156**, not 3,140
(``task-1-report.md``; re-confirmed directly against the read-only
reference DB this pass — see ``task-2-report.md``). Rather than propagate a
verified-wrong count into an identifier three tasks are going to read,
:func:`resolve_universe_variants` returns plain semantic keys (``artifact``
/ ``scopes`` / ``unrestricted``) with the real measured size available from
``len(variant.fips)`` — T4 (A3's emitter) owns picking the final
``universe_variant`` column strings and should use the measured number.

Usage::

    # one-time, pinned-toolchain only — materializes the registered parquet
    # sources this derivation reads (dist/ is gitignored; absent on a fresh box)
    mise exec -- uv run --frozen python tools/make_national_incidence_artifact.py --export-source

    # the skeleton itself (steps 1-2; prints sizes, computes no measures)
    uv run python tools/make_national_incidence_artifact.py
"""

from __future__ import annotations

import argparse
import csv
import gzip
import hashlib
import io
import json
import math
import sqlite3
import sys
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import NamedTuple

from pyarrow import dataset as ds  # type: ignore[import-untyped]
from pyarrow import parquet as pq

_REPO_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(_REPO_ROOT / "src"))
sys.path.insert(0, str(Path(__file__).resolve().parent))

import make_data_artifacts  # type: ignore[import-not-found]  # noqa: E402
from build_reference_db import PINNED_SQLITE_VERSION  # type: ignore[import-not-found]  # noqa: E402

from babylon.data.reference_scope import (  # noqa: E402
    DEFAULT_SQLITE_PATH,
    _load_national_fips,
)

ENGINE_TERRITORIES_JSON = _REPO_ROOT / "src/babylon/data/game/us_county_territories.json"
CROSSWALK_CSV = _REPO_ROOT / "src/babylon/data/reference/national/county_fips_vintage_crosswalk.csv"
DIST_DIR = _REPO_ROOT / "dist/data-artifacts"
MANIFEST_PATH = _REPO_ROOT / "data-artifacts.yaml"

#: The four registered parquet sources this derivation reads (plan §0 input
#: table; data-artifacts.yaml:291-301, 469-489, 726-736).
SOURCE_TABLES: tuple[str, ...] = (
    "fact_census_poverty",
    "dim_race",
    "dim_county",
    "dim_poverty_category",
)

#: OQ11 pinned vintage: 2019.
TIME_ID = 23
#: 1 = B17001_001 "Total" (universe u); 2 = B17001_002 "below poverty" (b).
CATEGORY_IDS: tuple[int, ...] = (1, 2)
#: T, A(White), B(Black), C/AIAN(Indigenous), D(Asian), E(NHPI),
#: F(Some other race), G(Two or more races), H(White-non-Hispanic/settler
#: reference), I(Hispanic/Chicano) — **all ten race_id values (1-10)**.
#: WIDENED FROM THE PLAN'S LITERAL ``(1, 2, 3, 4, 9, 10)`` by a controller
#: ruling at T4: the plan's own guard register self-contradicts (G6 needs
#: the seven ``A..G`` parts race_id 2-8 never pulled under the narrower
#: filter). The single pyarrow pull carries all ten so G6's T==Σ(A..G)
#: exactness check runs on the real pulled cells; the pole derivation then
#: selects its B/C/I/H subset downstream (see module docstring's pole-letter
#: trap note and :data:`POLE_RACE_ID`).
RACE_IDS: tuple[int, ...] = (1, 2, 3, 4, 5, 6, 7, 8, 9, 10)


class ArtifactGenerationError(Exception):
    """A generation or verification step failed loudly. No fallback."""


class SourceProvenanceError(ArtifactGenerationError):
    """Step 2: a source parquet's sha256 does not match the
    data-artifacts.yaml pin, or the source/manifest entry is missing."""


class UnpinnedToolchainError(ArtifactGenerationError):
    """``--export-source`` was invoked off the pinned SQLite runtime."""


def _sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _load_manifest_pins(names: tuple[str, ...], manifest_path: Path) -> dict[str, str]:
    """Read sha256 pins for ``names`` directly out of ``data-artifacts.yaml``
    (or a test double with the same ``artifacts: [{name, sha256}, ...]``
    shape) — no hand-typed duplicate hash strings in this module (the same
    "never hand-type a sha256" discipline
    ``docs/how-to/reference-data-pipeline.rst:63-65`` states for newly
    registered artifacts, applied here to reading already-registered ones).

    :raises ArtifactGenerationError: any requested ``name`` has no manifest entry.
    """
    import yaml

    manifest = yaml.safe_load(manifest_path.read_text(encoding="utf-8"))
    by_name = {entry["name"]: entry for entry in manifest["artifacts"]}
    missing = [name for name in names if name not in by_name]
    if missing:
        msg = f"{manifest_path}: no artifact entry for {missing}"
        raise ArtifactGenerationError(msg)
    return {name: str(by_name[name]["sha256"]) for name in names}


def export_source_tables(
    sqlite_path: Path = DEFAULT_SQLITE_PATH,
    *,
    dist_dir: Path = DIST_DIR,
    tables: tuple[str, ...] = SOURCE_TABLES,
) -> dict[str, Path]:
    """Step 1: ``--export-source`` mode.

    Opens the reference DB read-only (``mode=ro``, ``uri=True``) and calls
    ``make_data_artifacts.export_table_parquet`` — the ONLY sanctioned
    parquet-writing path — once per table in ``tables``. This PRODUCES the
    registered parquet sources; it never reads them for derivation (that's
    every other function in this module, all of which read only the
    resulting parquet files).

    :raises UnpinnedToolchainError: the runtime SQLite isn't the pinned
        version — an off-pin export risks bytes that don't match the
        registered sha256 pins.
    :raises ArtifactGenerationError: ``sqlite_path`` does not exist.
    """
    if sqlite3.sqlite_version != PINNED_SQLITE_VERSION:
        msg = (
            f"--export-source requires the pinned SQLite runtime "
            f"({PINNED_SQLITE_VERSION}); got {sqlite3.sqlite_version}. Run inside "
            "`mise exec -- uv run --frozen python ...` (docs/how-to/reference-data-pipeline.rst) — "
            "an off-pin export risks producing bytes that don't match the registered "
            "data-artifacts.yaml sha256 pins."
        )
        raise UnpinnedToolchainError(msg)
    if not sqlite_path.exists():
        msg = f"reference DB not found: {sqlite_path}"
        raise ArtifactGenerationError(msg)

    conn = sqlite3.connect(f"file:{sqlite_path}?mode=ro", uri=True)
    out_paths: dict[str, Path] = {}
    try:
        for table in tables:
            dest = dist_dir / f"{table}.parquet"
            rows, size = make_data_artifacts.export_table_parquet(conn, table, dest)
            sha = _sha256_file(dest)
            print(
                f"[national-incidence] exported {table}: {rows} rows, {size} bytes, "
                f"sha256={sha} -> {dest}"
            )
            out_paths[table] = dest
    finally:
        conn.close()
    return out_paths


def verify_source_provenance(
    dist_dir: Path = DIST_DIR,
    *,
    manifest_path: Path = MANIFEST_PATH,
    tables: tuple[str, ...] = SOURCE_TABLES,
) -> dict[str, Path]:
    """Step 2: hard-fail on provenance drift.

    Computes each source parquet's sha256 and compares it to the
    ``data-artifacts.yaml`` pin BEFORE any derivation read. Mismatch or
    absence raises :class:`SourceProvenanceError`, loud, no fallback — no
    "proceed with a warning" path exists.
    """
    pins = _load_manifest_pins(tables, manifest_path)
    verified: dict[str, Path] = {}
    for table in tables:
        path = dist_dir / f"{table}.parquet"
        if not path.exists():
            msg = (
                f"{table}: source parquet missing at {path} — run --export-source "
                "first (dist/ is gitignored; it is absent on a fresh box)"
            )
            raise SourceProvenanceError(msg)
        actual_sha = _sha256_file(path)
        if actual_sha != pins[table]:
            msg = (
                f"{table}: sha256 mismatch — manifest pin {pins[table]}, "
                f"actual {actual_sha} (data-artifacts.yaml drift or a corrupted/stale "
                f"{path})"
            )
            raise SourceProvenanceError(msg)
        verified[table] = path
    return verified


def _load_county_fips_map(county_parquet: Path) -> dict[int, str]:
    """``county_id -> fips`` from the sha-verified ``dim_county`` parquet."""
    table = pq.read_table(county_parquet, columns=["county_id", "fips"])
    return dict(zip(table["county_id"].to_pylist(), table["fips"].to_pylist(), strict=True))


class PovertyCell(NamedTuple):
    """One filtered ``fact_census_poverty`` row, FIPS-resolved."""

    fips: str
    category_id: int
    race_id: int
    person_count: int


def read_filtered_poverty_cells(
    poverty_parquet: Path,
    county_parquet: Path,
    *,
    time_id: int = TIME_ID,
    category_ids: tuple[int, ...] = CATEGORY_IDS,
    race_ids: tuple[int, ...] = RACE_IDS,
) -> tuple[PovertyCell, ...]:
    """Step 3: filtered pyarrow read.

    Pushes the ``time_id``/``category_id``/``race_id`` predicate into the
    dataset scan (column projection + predicate pushdown) — never a bare
    ``fetchall``/``to_pandas`` of the full 26.5M-row table. ``fips`` is
    resolved via ``dim_county``.

    :raises ArtifactGenerationError: a ``county_id`` in the filtered cells
        has no ``dim_county`` row (join integrity failure — never silently
        dropped).
    """
    fips_by_county_id = _load_county_fips_map(county_parquet)

    dataset = ds.dataset(poverty_parquet, format="parquet")
    predicate = (
        (ds.field("time_id") == time_id)
        & ds.field("category_id").isin(list(category_ids))
        & ds.field("race_id").isin(list(race_ids))
    )
    table = dataset.to_table(
        columns=["county_id", "category_id", "race_id", "person_count"],
        filter=predicate,
    )

    cells: list[PovertyCell] = []
    for county_id, category_id, race_id, person_count in zip(
        table["county_id"].to_pylist(),
        table["category_id"].to_pylist(),
        table["race_id"].to_pylist(),
        table["person_count"].to_pylist(),
        strict=True,
    ):
        fips = fips_by_county_id.get(county_id)
        if fips is None:
            msg = (
                f"fact_census_poverty county_id={county_id} has no dim_county row "
                "(join integrity failure)"
            )
            raise ArtifactGenerationError(msg)
        cells.append(
            PovertyCell(
                fips=fips, category_id=category_id, race_id=race_id, person_count=person_count
            )
        )
    return tuple(cells)


class CrosswalkRow(NamedTuple):
    """One A1 row, as read back from the checked-in CSV. Mirrors
    ``tools/make_fips_vintage_crosswalk.py``'s ``CrosswalkRow`` shape."""

    fips_engine: str
    fips_acs2019: str
    relation: str
    vintage_note: str
    recoverable: bool


def load_crosswalk(csv_path: Path = CROSSWALK_CSV) -> tuple[CrosswalkRow, ...]:
    """A1, loaded from the checked-in CSV — **the authority** (T1,
    review-verified). Not re-derived from
    ``tools/make_fips_vintage_crosswalk.py``'s in-module ``CROSSWALK_ROWS``
    constant; this module reads the artifact, the way any other A1
    consumer would."""
    rows: list[CrosswalkRow] = []
    with csv_path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        for record in reader:
            rows.append(
                CrosswalkRow(
                    fips_engine=record["fips_engine"],
                    fips_acs2019=record["fips_acs2019"],
                    relation=record["relation"],
                    vintage_note=record["vintage_note"],
                    recoverable=record["recoverable"] == "true",
                )
            )
    return tuple(rows)


def resolve_query_fips(
    fips_engine: str, crosswalk_by_engine_fips: Mapping[str, CrosswalkRow]
) -> str:
    """A1 applied: which FIPS to actually query ``fact_census_poverty``
    under for one engine-universe county.

    Only a ``recoverable=True`` row substitutes a target (Bedford ``51515``
    -> ``51019``). Every other case — no crosswalk row at all, a
    ``DECLARED_HOLE`` row, or a non-recoverable ``SPLIT_UNRESOLVED`` /
    ``REORGANIZATION_UNRESOLVED`` row — queries its own native
    ``fips_engine``, which for the latter two simply returns no data.
    Classifying *why* that data is absent is T3b's job (G4/G5); this
    resolver computes nothing.
    """
    row = crosswalk_by_engine_fips.get(fips_engine)
    if row is not None and row.recoverable and row.fips_acs2019:
        return row.fips_acs2019
    return fips_engine


class UniverseVariant(NamedTuple):
    """One resolved universe: a semantic name + its FIPS membership."""

    name: str
    fips: frozenset[str]


def _artifact_universe(territories_json: Path = ENGINE_TERRITORIES_JSON) -> frozenset[str]:
    """The engine territory universe (``us_county_territories.json``)."""
    payload = json.loads(territories_json.read_text(encoding="utf-8"))
    return frozenset(county["fips"] for county in payload["counties"])


def _scopes_universe(sqlite_path: Path = DEFAULT_SQLITE_PATH) -> frozenset[str]:
    """The resolver universe. Reuses ``scopes._load_national_fips`` directly
    (DRY; T1's own precedent) — see module docstring for why this sqlite
    read is not the ``--from-sqlite`` circularity."""
    return _load_national_fips(sqlite_path)


def _unrestricted_universe(
    poverty_parquet: Path, county_parquet: Path, *, time_id: int = TIME_ID
) -> frozenset[str]:
    """The raw set of FIPS present in ``fact_census_poverty`` at
    ``time_id`` — whatever the data itself contains, no universe
    restriction applied."""
    fips_by_county_id = _load_county_fips_map(county_parquet)
    dataset = ds.dataset(poverty_parquet, format="parquet")
    table = dataset.to_table(columns=["county_id"], filter=ds.field("time_id") == time_id)
    county_ids = set(table["county_id"].to_pylist())
    return frozenset(
        fips_by_county_id[county_id] for county_id in county_ids if county_id in fips_by_county_id
    )


def resolve_universe_variants(
    *,
    territories_json: Path = ENGINE_TERRITORIES_JSON,
    sqlite_path: Path = DEFAULT_SQLITE_PATH,
    poverty_parquet: Path,
    county_parquet: Path,
    time_id: int = TIME_ID,
) -> tuple[UniverseVariant, ...]:
    """Step 4 (plan §3 step 1): the three declared FIPS sets.

    DECLARED_HOLE counties (``46102``, ``02158``) are **not** removed from
    any universe here — they remain ordinary members of ``artifact`` and
    ``scopes`` (both static county registries) and are naturally absent
    from ``unrestricted`` (zero ``fact_census_poverty`` rows at any FIPS
    they've held). G5 (T3b) requires them "present ... in every universe
    variant" for reconciliation; this function does not classify absence at
    all — see :func:`resolve_query_fips` for A1's actual application
    (which FIPS a downstream cell-pull queries under).
    """
    return (
        UniverseVariant("artifact", _artifact_universe(territories_json)),
        UniverseVariant("scopes", _scopes_universe(sqlite_path)),
        UniverseVariant(
            "unrestricted",
            _unrestricted_universe(poverty_parquet, county_parquet, time_id=time_id),
        ),
    )


# ---------------------------------------------------------------------------
# T3a — arithmetic-law guards: G1, G2, G6, G8 (plan §3 guard register).
# Each is a separate pure function; none is yet wired into a per-county
# measure pipeline (that lands with T3b/T4). G3/G4/G5 land with T3b; G7
# already shipped with T1 (tools/make_fips_vintage_crosswalk.py).
# ---------------------------------------------------------------------------


class AggregationCell(NamedTuple):
    """One ``(fips, universe_u, below_b)`` pair fed to a G1 pooled ratio.
    Callers exclude ``ZERO_DENOMINATOR`` cells (G2) before building this
    sequence — :func:`ratio_of_sums` has no zero-denominator policy of its
    own; that classification is :func:`classify_zero_denominator`'s job."""

    fips: str
    universe_u: int
    below_b: int


def ratio_of_sums(cells: Sequence[AggregationCell]) -> float:
    """G1: p̄/q̄ = Σb / Σu — the pooled rate is the ratio of the summed
    counts, **never** ``mean(rate_i)`` over per-county rates (that would
    silently overweight small-universe counties relative to their actual
    population share — plan §3 guard table, G1 row).

    :raises ArtifactGenerationError: Σu == 0 (no cells, or every cell has a
        zero universe) — undefined over an empty universe, never silently 0.0.
    """
    total_u = sum(cell.universe_u for cell in cells)
    total_b = sum(cell.below_b for cell in cells)
    if total_u == 0:
        msg = "ratio_of_sums: Σu == 0 — no defined ratio over an empty universe"
        raise ArtifactGenerationError(msg)
    return total_b / total_u


class ZeroDenominatorResult(NamedTuple):
    """G2's classification result. ``absence_class`` is ``"ZERO_DENOMINATOR"``
    when ``universe_u == 0``, else ``None`` (the caller applies its other
    absence-class rules — G4/G5, T3b). ``rate`` is the empty measure cell:
    ``None`` on zero-denominator, **never** a fabricated ``0.0`` — a ``0.0``
    on a diverging ramp renders *at the settler norm*, a fabricated data
    point (Constitution III.11)."""

    absence_class: str | None
    rate: float | None


def classify_zero_denominator(universe_u: int, below_b: int) -> ZeroDenominatorResult:
    """G2: ``u == 0`` ⇒ ``absence_class=ZERO_DENOMINATOR``, ``rate=None`` —
    all measure cells EMPTY, never zero (plan §3 guard table, G2 row; same
    error class as ``check:aggregation``'s "all-masked group input must
    yield None, never a fabricated 0.0").

    :raises ArtifactGenerationError: ``universe_u`` is negative (a malformed
        cell — a count can never be negative).
    """
    if universe_u < 0:
        msg = f"classify_zero_denominator: universe_u={universe_u} < 0 (malformed cell)"
        raise ArtifactGenerationError(msg)
    if universe_u == 0:
        return ZeroDenominatorResult(absence_class="ZERO_DENOMINATOR", rate=None)
    return ZeroDenominatorResult(absence_class=None, rate=below_b / universe_u)


#: The 7 mutually-exclusive census race/combination iterations that sum
#: exactly to T (the standard ACS detailed-table scheme). H (White-alone
#: non-Hispanic) and I (Hispanic, any race) are separate, overlapping
#: breakdowns — never part of this sum (G8 handles their overlap).
POLE_PART_LETTERS: tuple[str, ...] = ("A", "B", "C", "D", "E", "F", "G")


def assert_t_pole_exactness(
    fips: str,
    t: int,
    parts: Mapping[str, int],
    *,
    h: int,
    i: int,
) -> None:
    """G6: ``T == Σ(A..G)`` per county, **exact** equality — no tolerance
    window, ever (plan's F4: max residual 0 over 3,218 counties). Also
    enforces ``H <= A`` (H is the White-alone-non-Hispanic subset of A) and
    ``I <= T`` (I, Hispanic of any race, cannot exceed the county total).

    :raises ArtifactGenerationError: ``parts`` is missing one of the seven
        required letters, ``T != Σ(A..G)``, ``H > A``, or ``I > T``.
    """
    missing = [letter for letter in POLE_PART_LETTERS if letter not in parts]
    if missing:
        msg = f"{fips}: assert_t_pole_exactness missing pole parts {missing}"
        raise ArtifactGenerationError(msg)
    total_parts = sum(parts[letter] for letter in POLE_PART_LETTERS)
    if total_parts != t:
        residual = t - total_parts
        msg = (
            f"{fips}: T-pole exactness violated — T={t}, Σ(A..G)={total_parts}, "
            f"residual={residual} (exact equality required, no tolerance)"
        )
        raise ArtifactGenerationError(msg)
    if h > parts["A"]:
        msg = f"{fips}: H={h} exceeds A={parts['A']} (H must be the White-alone-non-Hispanic subset of A)"
        raise ArtifactGenerationError(msg)
    if i > t:
        msg = f"{fips}: I={i} exceeds T={t}"
        raise ArtifactGenerationError(msg)


def overlap_upper_bound(total_a: int, white_non_hispanic_h: int, hispanic_i: int) -> int:
    """G8: the disclosed, **never-subtracted** overlap bound — ``I - (A - H)``.
    ``(A - H)`` is the White-alone-Hispanic subgroup; subtracting it from
    ``I`` (all Hispanic persons, any race) bounds how many Hispanic persons
    are also counted in a non-white racial pole (B..G) — a double-count
    risk this function only DISCLOSES. No caller may net this out of any
    pole sum (that is exactly what G8 forbids — plan §3 guard table, G8 row).

    :raises ArtifactGenerationError: ``white_non_hispanic_h > total_a`` (H
        must be a subset of A), or the resulting bound is negative (fewer
        Hispanic persons than the White-Hispanic subgroup alone —
        impossible for valid census data).
    """
    if white_non_hispanic_h > total_a:
        msg = f"overlap_upper_bound: H={white_non_hispanic_h} > A={total_a} (H must be ⊆ A)"
        raise ArtifactGenerationError(msg)
    white_hispanic = total_a - white_non_hispanic_h
    bound = hispanic_i - white_hispanic
    if bound < 0:
        msg = (
            f"overlap_upper_bound: computed bound {bound} < 0 — I={hispanic_i} implies "
            f"fewer Hispanic persons than the White-Hispanic subgroup ({white_hispanic})"
        )
        raise ArtifactGenerationError(msg)
    return bound


# ---------------------------------------------------------------------------
# T3b — absence + small-count guards: G3, G4, G5 (plan §3 guard register).
# Each is a separate pure function; none is yet wired into a per-county
# measure pipeline (that lands with T4). G1/G2/G6/G8 shipped with T3a; G7
# already shipped with T1.
# ---------------------------------------------------------------------------


#: G3's declared damping measure. **Derivation** (ADR172 ruling 5, the
#: standing no-imposed-forms line — ``damp`` is a MEASURE derived from
#: counting statistics, never a stipulated curve shape): a per-county rate
#: built from a universe of size ``u`` is, to first order, a proportion
#: estimate over ``u`` independent Bernoulli trials. Its own sampling noise
#: has a coefficient of variation (relative standard error) of
#: ``1/sqrt(u)`` — the standard result for a Poisson/binomial count
#: (SD ≈ sqrt(u), so SD/u == 1/sqrt(u)). That fraction is exactly how much
#: of the raw deviation ``|w|`` is attributable to the count's own sampling
#: noise rather than real signal; ``damp(u) = 1 - 1/sqrt(u)`` is the
#: complementary reliability fraction — the share of ``|w|`` trusted as
#: signal. It is undefined at ``u=0`` (G2's ZERO_DENOMINATOR territory
#: fires first — there is no rate, let alone a reliability fraction, over
#: an empty universe), strictly increasing in ``u`` (``d/du(1 - u^-0.5) =
#: 0.5 u^-1.5 > 0`` for all ``u > 0`` — monotone by construction, not by
#: inspection), and approaches 1 in the limit (an infinite count carries no
#: relative sampling noise).
def damp(universe_u: int) -> float:
    """G3: ``damp(u) = 1 - 1/sqrt(u)`` — see the module-level derivation
    comment immediately above this function for why this shape, not an
    imposed sigmoid.

    :raises ArtifactGenerationError: ``universe_u <= 0`` — undefined at 0
        (G2 fires first), and a negative count is malformed.
    """
    if universe_u <= 0:
        msg = f"damp: universe_u={universe_u} <= 0 — damp(0) is undefined (G2 fires first)"
        raise ArtifactGenerationError(msg)
    return 1.0 - (1.0 / math.sqrt(universe_u))


class DampedSigma(NamedTuple):
    """G3's output: ``sigma_damped`` plus the ``damping_weight`` actually
    applied, published separately so the damping is auditable per row
    rather than baked invisibly into ``sigma_damped`` alone (T3b step 2)."""

    sigma_damped: float
    damping_weight: float


def compute_damped_sigma(w: float, universe_u: int) -> DampedSigma:
    """``sigma_damped = |w| * damp(u)`` (plan §3 step 8). Calls the module
    global ``damp`` (not a bound alias), so patching ``damp`` on this
    module reaches every caller.

    :raises ArtifactGenerationError: propagated from :func:`damp` when
        ``universe_u <= 0``.
    """
    weight = damp(universe_u)
    return DampedSigma(sigma_damped=abs(w) * weight, damping_weight=weight)


#: G4's material relation (Aleksandrov Test): why SUPPRESSED vs PRESENT is
#: not decorative. ``fact_census_poverty.person_count`` is **NOT NULL**
#: (plan §0, verified against the live schema) — an ACS-suppressed cell can
#: therefore ONLY surface in this table as the same literal 0 a genuine
#: zero-below-poverty count would produce; there is no suppression flag
#: this module can read. Trusting every reported zero as real would
#: silently launder small-sample statistical unreliability into a "this
#: county has zero deprivation" claim on exactly the axis this artifact
#: exists to measure honestly — the III.11 failure this guard prevents.
#: **This module declares its own statistical-plausibility policy** (below,
#: :func:`classify_suppression`) rather than claiming to reproduce an
#: internal Census suppression rule this data source does not expose.
#:
#: **Known limitation (T4 review obligation):** :data:`SUPPRESSION_REFERENCE_RATE`
#: (0.10) is applied FLAT across all four poles (B, C, I, H), even though the
#: true poverty-incidence rate differs by pole — the whole point of this
#: artifact is that it does (F1: q̄ ≈ 0.219 on the oppressed B+C+I pool vs
#: p̄ ≈ 0.096 on the settler H pool). This is a **deliberate simplification**
#: to avoid circularity with G1's own pooled ratios: G4 (step 4, this
#: classifier) must run BEFORE G1 (step 5) computes p̄/q̄, so it cannot use
#: per-pole rates without either a forward reference or a two-pass ordering
#: this module does not implement. **The simplification has directional
#: consequences, disclosed rather than hidden**: on the oppressed poles
#: (whose true rate ≈0.22 is well above the flat 0.10 reference), a genuine
#: zero is LESS surprising than the flat-rate model believes, so this
#: classifier UNDER-FLAGS suppression there (some true suppressions read as
#: PRESENT). On the settler pole (true rate ≈0.10, close to the reference),
#: the flat rate is roughly accurate; on any pole whose true rate is well
#: BELOW 0.10, the classifier would OVER-FLAG suppression (genuine zeros
#: misread as SUPPRESSED). Net: this policy is conservative in the wrong
#: direction for the axis the artifact exists to measure honestly, and a
#: future revision could re-run G4 in a second pass after G1 with per-pole
#: reference rates — not attempted here to keep T4's step ordering linear.
G4_MATERIAL_RELATION = (
    "fact_census_poverty.person_count is NOT NULL, so an ACS-suppressed cell can only "
    "surface as a literal 0 -- identical to a genuine zero-below-poverty count. This module "
    "classifies SUPPRESSED vs PRESENT by a declared statistical-plausibility policy (see "
    "classify_suppression's docstring), never by trusting every reported zero as real, "
    "because a silently-trusted fabricated zero on this axis is exactly the III.11 failure "
    "the national-incidence artifact exists to prevent. KNOWN LIMITATION: the reference rate "
    "(0.10) is applied flat across all four poles although true rates differ by pole (F1: "
    "q̄≈ 0.22 oppressed vs p̄≈ 0.10 settler) -- a deliberate simplification to avoid "
    "circularity with G1's own pooled ratios (G4/step 4 runs before G1/step 5 computes them). "
    "Directional consequence: this UNDER-FLAGS suppression on the oppressed poles (true rate "
    "well above the flat reference makes a genuine zero less surprising than modeled) and "
    "would OVER-FLAG on any pole whose true rate is well below the flat reference."
)

#: The reference poverty-incidence rate used ONLY to test the statistical
#: plausibility of an exact-zero below-poverty count (G4). This is a
#: declared POLICY PARAMETER, not the pipeline's own measured p̄/q̄ (G1,
#: step 5) — G4 (step 4) must run BEFORE G1 computes those, so it cannot
#: depend on them without a circular ordering.
SUPPRESSION_REFERENCE_RATE = 0.10
#: The conventional "surprising" significance threshold: a genuine-zero
#: event with probability below this under the reference rate is treated
#: as more likely explained by suppression than by chance.
SUPPRESSION_IMPLAUSIBILITY_ALPHA = 0.05


def classify_suppression(universe_u: int, below_b: int) -> str:
    """G4: distinguishes a genuine small-count zero (``"PRESENT"``) from a
    suppression-consistent zero (``"SUPPRESSED"``) for a cell where
    ``universe_u > 0`` and ``below_b == 0`` (``universe_u == 0`` is G2's
    ZERO_DENOMINATOR territory; a nonzero ``below_b`` needs no suppression
    judgment at all).

    **The declared rule**: under the reference rate ``r`` =
    :data:`SUPPRESSION_REFERENCE_RATE`, modeling each of the ``u`` persons'
    poverty status as an independent Bernoulli trial, the probability of a
    genuine zero count is ``(1 - r) ** u``. When that probability falls
    below :data:`SUPPRESSION_IMPLAUSIBILITY_ALPHA`, the observed exact zero
    would be statistically surprising even under complete independence —
    exactly the pattern ACS small-count disclosure-avoidance is documented
    to produce (see :data:`G4_MATERIAL_RELATION`) — so this cell is
    classified SUPPRESSED rather than trusted. Below the threshold, a
    genuine zero is unremarkable and the cell is PRESENT.

    :raises ArtifactGenerationError: ``universe_u <= 0`` (G2's territory),
        or ``below_b != 0`` (out of this function's domain).
    """
    if universe_u <= 0:
        msg = f"classify_suppression: universe_u={universe_u} <= 0 is G2's territory, not G4's"
        raise ArtifactGenerationError(msg)
    if below_b != 0:
        msg = f"classify_suppression: below_b={below_b} != 0 — only exact-zero cells need a suppression judgment"
        raise ArtifactGenerationError(msg)
    probability_of_genuine_zero = (1.0 - SUPPRESSION_REFERENCE_RATE) ** universe_u
    if probability_of_genuine_zero < SUPPRESSION_IMPLAUSIBILITY_ALPHA:
        return "SUPPRESSED"
    return "PRESENT"


class PoleCellPair(NamedTuple):
    """One (fips, pole)'s already-joined universe/below-poverty counts —
    the shape :func:`classify_absence` consumes. A missing pair (``None``)
    for a (fips, pole) in the declared universe means no
    ``fact_census_poverty`` rows exist at all for that cell — ROW_ABSENT,
    never an imputed count."""

    universe_u: int
    below_b: int


class AbsenceClassification(NamedTuple):
    """G4's full per-cell result: which A2 absence class the cell belongs
    to, plus its rate — ``None`` for every non-PRESENT class (G2's
    "absence is a value, never 0.0" law applies identically to SUPPRESSED
    and ROW_ABSENT, not just ZERO_DENOMINATOR)."""

    absence_class: str
    rate: float | None


def classify_absence(
    cell: PoleCellPair | None, *, fips: str, declared_hole_fips: frozenset[str]
) -> AbsenceClassification:
    """The full per-(fips, pole) absence classification: DECLARED_HOLE (A1;
    overrides everything else — a declared-hole fips is never trusted even
    if a malformed cell somehow carried data for it), ROW_ABSENT (``cell``
    is ``None``), ZERO_DENOMINATOR (G2, reused), SUPPRESSED/PRESENT (G4,
    :func:`classify_suppression`), or PRESENT (a normal nonzero cell).

    :raises ArtifactGenerationError: propagated from
        :func:`classify_zero_denominator` or :func:`classify_suppression`.
    """
    if fips in declared_hole_fips:
        return AbsenceClassification(absence_class="DECLARED_HOLE", rate=None)
    if cell is None:
        return AbsenceClassification(absence_class="ROW_ABSENT", rate=None)
    zero_denom = classify_zero_denominator(cell.universe_u, cell.below_b)
    if zero_denom.absence_class == "ZERO_DENOMINATOR":
        return AbsenceClassification(absence_class="ZERO_DENOMINATOR", rate=None)
    if cell.below_b == 0:
        label = classify_suppression(cell.universe_u, cell.below_b)
        rate = zero_denom.rate if label == "PRESENT" else None
        return AbsenceClassification(absence_class=label, rate=rate)
    return AbsenceClassification(absence_class="PRESENT", rate=zero_denom.rate)


#: G5's declared absence-class taxonomy — every (fips, pole) cell in the
#: declared universe lands in exactly one of these (PRESENT included so the
#: reconciliation is exhaustive: presence + absence == universe_size).
ABSENCE_CLASSES: tuple[str, ...] = (
    "PRESENT",
    "ZERO_DENOMINATOR",
    "ROW_ABSENT",
    "DECLARED_HOLE",
    "SUPPRESSED",
)


class AbsenceReconciliation(NamedTuple):
    """G5's per-class county counts plus the exact reconciliation."""

    counts_by_class: Mapping[str, int]
    counties_present: int
    counties_absent: int
    universe_size: int


def reconcile_absence_counts(
    absence_classes: Sequence[str], *, universe_size: int
) -> AbsenceReconciliation:
    """G5: honest absence accounting. ``counties_present`` (the ``PRESENT``
    label) + ``counties_absent`` (every other label in
    :data:`ABSENCE_CLASSES`) must equal ``universe_size`` EXACTLY — no
    tolerance, no dropped class (F3: the real run budgets 466 pole-cells +
    14/16 whole-county, not 14/16 alone — this function's law is generic;
    it must reconcile exactly regardless of which classes carry the counts).

    :raises ArtifactGenerationError: any label outside
        :data:`ABSENCE_CLASSES`, or the reconciled total does not equal
        ``universe_size``.
    """
    unknown = sorted(set(absence_classes) - set(ABSENCE_CLASSES))
    if unknown:
        msg = f"reconcile_absence_counts: unknown absence class(es) {unknown}"
        raise ArtifactGenerationError(msg)
    counts: dict[str, int] = dict.fromkeys(ABSENCE_CLASSES, 0)
    for label in absence_classes:
        counts[label] += 1
    counties_present = counts["PRESENT"]
    counties_absent = sum(counts[label] for label in ABSENCE_CLASSES if label != "PRESENT")
    total = counties_present + counties_absent
    if total != universe_size:
        msg = (
            f"reconcile_absence_counts: counties_present({counties_present}) + "
            f"counties_absent({counties_absent}) = {total} != universe_size={universe_size}"
        )
        raise ArtifactGenerationError(msg)
    return AbsenceReconciliation(
        counts_by_class=counts,
        counties_present=counties_present,
        counties_absent=counties_absent,
        universe_size=universe_size,
    )


#: G5's Pine Ridge invariant. 46102 (Oglala Lakota) is a permanent
#: DECLARED_HOLE (A1, T1) — zero fact_census_poverty rows at every
#: time_id; its retired predecessor 46113 (Shannon County) carries rows
#: only 2010-2014, stale before the pinned 2019 vintage (OQ11). Never
#: imputed.
PINE_RIDGE_FIPS = "46102"
PINE_RIDGE_RETIRED_PREDECESSOR_FIPS = "46113"


def assert_no_pine_ridge_imputation(
    fips: str, absence_class: str, *, source_fips: str | None = None
) -> None:
    """G5's Pine Ridge leg. Out of scope (no-op) for any ``fips`` other
    than :data:`PINE_RIDGE_FIPS`.

    :raises ArtifactGenerationError: ``fips == PINE_RIDGE_FIPS`` and either
        ``source_fips == PINE_RIDGE_RETIRED_PREDECESSOR_FIPS`` (an
        imputation attempt from the retired predecessor), or
        ``absence_class != "DECLARED_HOLE"`` (Pine Ridge must classify
        DECLARED_HOLE in every universe variant).
    """
    if fips != PINE_RIDGE_FIPS:
        return
    if source_fips == PINE_RIDGE_RETIRED_PREDECESSOR_FIPS:
        msg = (
            f"{fips}: Pine Ridge imputed from its retired predecessor "
            f"{PINE_RIDGE_RETIRED_PREDECESSOR_FIPS} — never permitted (G5)"
        )
        raise ArtifactGenerationError(msg)
    if absence_class != "DECLARED_HOLE":
        msg = (
            f"{fips}: Pine Ridge must classify DECLARED_HOLE in every universe variant, "
            f"got {absence_class!r}"
        )
        raise ArtifactGenerationError(msg)


# ---------------------------------------------------------------------------
# T4 — wire every guard into the full per-county pipeline (plan §3 steps
# 3-9) and emit A2 + A3 deterministically (step 10). Pole map first, then
# the join/classify/pool/measure passes, then the two CSV writers.
# ---------------------------------------------------------------------------

#: The four charter poles this artifact can reach (plan §0's pole map,
#: F6). **Pole-letter trap** (do not swap): Chicano is census race_id 10
#: (``I``, Hispanic any race); Indigenous is census race_id 4 (``C``,
#: AIAN). B (Black) is race_id 3; the settler-reference pole H
#: (White-non-Hispanic) is race_id 9.
POLE_LETTERS: tuple[str, ...] = ("B", "C", "I", "H")
OPPRESSED_POLE_LETTERS: tuple[str, ...] = ("B", "C", "I")
SETTLER_POLE_LETTER = "H"
POLE_RACE_ID: dict[str, int] = {"B": 3, "C": 4, "I": 10, "H": 9}
POLE_ROLE: dict[str, str] = {
    "B": "oppressed",
    "C": "oppressed",
    "I": "oppressed",
    "H": "settler_reference",
}
#: The county total (T, race_id=1) — ``U(i)`` in the proposal, the ACS
#: poverty-universe denominator every pole's per-capita measures divide by.
TOTAL_RACE_ID = 1
#: race_id -> G6's ``POLE_PART_LETTERS`` key, the seven mutually-exclusive
#: "alone" race categories (2=A White, 3=B Black, 4=C AIAN, 5=D Asian,
#: 6=E NHPI, 7=F Some other race, 8=G Two or more races).
RACE_ID_TO_PART_LETTER: dict[int, str] = {2: "A", 3: "B", 4: "C", 5: "D", 6: "E", 7: "F", 8: "G"}
#: race_id=2, "A" White alone — G8's ``total_a`` input.
WHITE_ALONE_RACE_ID = 2
#: A2's ``fips_source_vintage`` value when no crosswalk row was applied
#: (the overwhelming common case — only Bedford ``51515`` substitutes).
NATIVE_VINTAGE_LABEL = "native"
#: Pinned float format for every measure column (brief step 2) — an ad-hoc
#: ``repr``/``str`` is a cross-run byte hazard.
_FLOAT_FMT = "{:.9f}"
#: The sentinel ``pole`` value on A3's B+C+I combined row (never one of the
#: four charter pole letters, so a reader can never confuse it with a row
#: for an actual pole).
POOLED_POLE_LABEL = "POOLED"

A2_OUTPUT_PATH = (
    _REPO_ROOT / "src/babylon/data/reference/national/national_incidence_county_pole.csv.gz"
)
A3_OUTPUT_PATH = _REPO_ROOT / "src/babylon/data/reference/national/national_reproduction_floor.csv"


def _require_int(value: int | None, *, context: str) -> int:
    """Every PRESENT-classified cell's ``universe_u``/``below_b`` must be a
    real int by construction (G2/G4 only leave ``None`` on non-PRESENT
    rows). A ``None`` here is an internal wiring bug, not a data condition.

    :raises ArtifactGenerationError: ``value is None``.
    """
    if value is None:
        msg = f"_require_int: unexpected None ({context})"
        raise ArtifactGenerationError(msg)
    return value


def build_county_race_totals(
    cells: Sequence[PovertyCell],
) -> dict[str, dict[int, PoleCellPair]]:
    """Step 3's precondition: pairs each (fips, race_id)'s ``category_id=1``
    (universe) and ``category_id=2`` (below-poverty) rows into one
    :class:`PoleCellPair` — every downstream guard (G6's per-county
    exactness, G4's per-pole classification) reads this joined shape, never
    raw ``(category_id, race_id, person_count)`` rows.

    :raises ArtifactGenerationError: a pulled cell has a ``category_id``
        outside ``{1, 2}`` (the pinned filter never requests any other), a
        ``(fips, race_id)`` pair has only one of the two category rows
        (a malformed pull — the pinned filter always requests both
        together, so this should never fire on real data), or a joined
        pair has ``below_b > universe_u`` (the below-poverty count cannot
        exceed the universe it is drawn from — a plain data-integrity
        check, not a numbered G1-G8 guard).
    """
    universe_by_key: dict[tuple[str, int], int] = {}
    below_by_key: dict[tuple[str, int], int] = {}
    for cell in cells:
        key = (cell.fips, cell.race_id)
        if cell.category_id == 1:
            universe_by_key[key] = cell.person_count
        elif cell.category_id == 2:
            below_by_key[key] = cell.person_count
        else:
            msg = f"{cell.fips}: unexpected category_id={cell.category_id} (race_id={cell.race_id})"
            raise ArtifactGenerationError(msg)

    keys = set(universe_by_key) | set(below_by_key)
    mismatched = sorted(k for k in keys if k not in universe_by_key or k not in below_by_key)
    if mismatched:
        msg = (
            f"build_county_race_totals: {len(mismatched)} (fips, race_id) pair(s) missing one "
            f"category row: {mismatched[:5]}"
        )
        raise ArtifactGenerationError(msg)

    out: dict[str, dict[int, PoleCellPair]] = {}
    # sorted() defensively — plain set iteration is hash-seed-order
    # sensitive across process invocations, and this loop's insertion order
    # becomes the output dict's key order (near-zero cost at this size).
    for fips, race_id in sorted(keys):
        universe_u = universe_by_key[(fips, race_id)]
        below_b = below_by_key[(fips, race_id)]
        if below_b > universe_u:
            msg = (
                f"{fips}: race_id={race_id} below_b={below_b} exceeds universe_u={universe_u} "
                "(malformed cell -- below-poverty count cannot exceed the universe)"
            )
            raise ArtifactGenerationError(msg)
        out.setdefault(fips, {})[race_id] = PoleCellPair(universe_u=universe_u, below_b=below_b)
    return out


def run_t_pole_exactness_for_county(fips: str, by_race: Mapping[int, PoleCellPair]) -> None:
    """Step 3, wired: G6 on one county's already-joined race totals,
    checked on the **universe (category_id=1) counts only** — F4 measured
    the exact-equality invariant at category_id=1; this module makes no
    claim about category_id=2 (below-poverty) partition exactness (ACS
    disclosure-avoidance perturbation may not preserve it).

    :raises ArtifactGenerationError: any of race_ids 1-10 (T, A-G, H, I)
        missing from ``by_race``, or propagated from
        :func:`assert_t_pole_exactness`.
    """
    required_race_ids = (
        TOTAL_RACE_ID,
        *RACE_ID_TO_PART_LETTER,
        POLE_RACE_ID["H"],
        POLE_RACE_ID["I"],
    )
    missing = [race_id for race_id in required_race_ids if race_id not in by_race]
    if missing:
        msg = f"{fips}: run_t_pole_exactness_for_county missing race_id(s) {missing} (need T, A-G, H, I)"
        raise ArtifactGenerationError(msg)
    parts = {
        letter: by_race[race_id].universe_u for race_id, letter in RACE_ID_TO_PART_LETTER.items()
    }
    assert_t_pole_exactness(
        fips,
        t=by_race[TOTAL_RACE_ID].universe_u,
        parts=parts,
        h=by_race[POLE_RACE_ID["H"]].universe_u,
        i=by_race[POLE_RACE_ID["I"]].universe_u,
    )


class ClassifiedCell(NamedTuple):
    """Steps 1(A1)+3(G6)+4(G4)'s combined output for one (engine_fips,
    pole): which A1-resolved fips the data was actually queried under, the
    absence class, the rate (``None`` off-PRESENT), and the raw
    universe_u/below_b (``None`` off-PRESENT — G2's "absence is a value,
    never 0.0" law)."""

    engine_fips: str
    query_fips: str
    pole: str
    absence_class: str
    rate: float | None
    universe_u: int | None
    below_b: int | None
    fips_source_vintage: str


def classify_universe_poles(
    universe_fips: frozenset[str],
    by_race_by_query_fips: Mapping[str, Mapping[int, PoleCellPair]],
    crosswalk_rows: Sequence[CrosswalkRow],
) -> tuple[ClassifiedCell, ...]:
    """Steps 1(A1 applied)+3(G6)+4(G4): for every ``(engine_fips, pole)`` in
    the declared universe, resolve the query fips (A1 — only Bedford
    ``51515`` substitutes), run G6's per-county exactness check once (when
    data exists and the county isn't a declared hole), then classify each
    of the 4 poles' absence (G4, reusing G2). Runs G5's Pine Ridge leg on
    every cell as a standing regression guard. Sorted ``(engine_fips,
    pole)`` for determinism.
    """
    declared_hole_fips = frozenset(
        row.fips_engine for row in crosswalk_rows if row.relation == "DECLARED_HOLE"
    )
    crosswalk_by_engine = {row.fips_engine: row for row in crosswalk_rows}

    out: list[ClassifiedCell] = []
    for engine_fips in sorted(universe_fips):
        query_fips = resolve_query_fips(engine_fips, crosswalk_by_engine)
        crosswalk_row = crosswalk_by_engine.get(engine_fips)
        vintage = (
            crosswalk_row.vintage_note
            if crosswalk_row is not None and crosswalk_row.recoverable
            else NATIVE_VINTAGE_LABEL
        )
        by_race = by_race_by_query_fips.get(query_fips)
        if by_race is not None and engine_fips not in declared_hole_fips:
            run_t_pole_exactness_for_county(query_fips, by_race)
        for pole in POLE_LETTERS:
            cell = by_race.get(POLE_RACE_ID[pole]) if by_race is not None else None
            classification = classify_absence(
                cell, fips=engine_fips, declared_hole_fips=declared_hole_fips
            )
            assert_no_pine_ridge_imputation(
                engine_fips, classification.absence_class, source_fips=query_fips
            )
            out.append(
                ClassifiedCell(
                    engine_fips=engine_fips,
                    query_fips=query_fips,
                    pole=pole,
                    absence_class=classification.absence_class,
                    rate=classification.rate,
                    universe_u=cell.universe_u if cell is not None else None,
                    below_b=cell.below_b if cell is not None else None,
                    fips_source_vintage=vintage,
                )
            )
    return tuple(out)


class PooledRatios(NamedTuple):
    """G1's step-5 output: the two pooled reference rates every present
    cell's step-7 measures are computed against."""

    p_bar: float
    q_bar: float


def compute_pooled_ratios(classified: Sequence[ClassifiedCell]) -> PooledRatios:
    """Step 5: G1 ratio-of-sums, pooled over PRESENT cells only —
    ``p̄`` over the settler (H) pole, ``q̄`` over the B+C+I oppressed poles
    combined (the ruled partition, OQ1). Never mean-of-ratios (G1's law).

    :raises ArtifactGenerationError: propagated from :func:`ratio_of_sums`
        when either pool has zero PRESENT cells.
    """
    settler_cells = tuple(
        AggregationCell(
            fips=c.engine_fips,
            universe_u=_require_int(c.universe_u, context="settler pool"),
            below_b=_require_int(c.below_b, context="settler pool"),
        )
        for c in classified
        if c.pole == SETTLER_POLE_LETTER and c.absence_class == "PRESENT"
    )
    oppressed_cells = tuple(
        AggregationCell(
            fips=c.engine_fips,
            universe_u=_require_int(c.universe_u, context="oppressed pool"),
            below_b=_require_int(c.below_b, context="oppressed pool"),
        )
        for c in classified
        if c.pole in OPPRESSED_POLE_LETTERS and c.absence_class == "PRESENT"
    )
    return PooledRatios(p_bar=ratio_of_sums(settler_cells), q_bar=ratio_of_sums(oppressed_cells))


def compute_w(below_b: int, universe_u: int, p_bar: float) -> float:
    """Step 7: ``w = (b - u·p̄) / (b + u·p̄)`` — the signed witness (proposal
    §2.1), applied uniformly to every pole (A2's schema note: the settler
    pole's own ``w``/masses fall out of the same arithmetic rather than a
    special case).

    :raises ArtifactGenerationError: ``b + u·p̄ == 0`` (only possible if
        both ``below_b == 0`` and ``p_bar == 0`` — never on real data with
        ``p_bar > 0``, guarded explicitly rather than left to divide-by-zero).
    """
    denominator = below_b + universe_u * p_bar
    if denominator == 0:
        msg = f"compute_w: denominator b + u·p̄ == 0 (b={below_b}, u={universe_u}, p̄={p_bar})"
        raise ArtifactGenerationError(msg)
    return (below_b - universe_u * p_bar) / denominator


class CountyPoleRow(NamedTuple):
    """A2's row shape — one ``(fips, pole)`` cell. Every measure column is
    ``None`` (never a fabricated 0.0) when ``absence_class != "PRESENT"``."""

    fips: str
    pole: str
    pole_role: str
    universe_u: int | None
    below_b: int | None
    rate: float | None
    w: float | None
    sigma_damped: float | None
    damping_weight: float | None
    mass_vs_settler_norm: float | None
    mass_vs_demonstrated_floor: float | None
    lambda_per_capita: float | None
    omega_hat_per_capita: float | None
    absence_class: str
    fips_source_vintage: str


def _measure_row(
    classified_cell: ClassifiedCell,
    by_race: Mapping[int, PoleCellPair] | None,
    *,
    p_bar: float,
    q_bar: float,
) -> CountyPoleRow:
    """Steps 7-8 for one classified cell: G3's damped sigma plus the two
    mass columns and their per-capita intensities, gated by G2/G4's
    classification — empty measure cells off-PRESENT (plan's A2 note)."""
    if classified_cell.absence_class != "PRESENT":
        return CountyPoleRow(
            fips=classified_cell.engine_fips,
            pole=classified_cell.pole,
            pole_role=POLE_ROLE[classified_cell.pole],
            universe_u=None,
            below_b=None,
            rate=None,
            w=None,
            sigma_damped=None,
            damping_weight=None,
            mass_vs_settler_norm=None,
            mass_vs_demonstrated_floor=None,
            lambda_per_capita=None,
            omega_hat_per_capita=None,
            absence_class=classified_cell.absence_class,
            fips_source_vintage=classified_cell.fips_source_vintage,
        )

    universe_u = _require_int(classified_cell.universe_u, context="PRESENT cell")
    below_b = _require_int(classified_cell.below_b, context="PRESENT cell")
    if by_race is None or TOTAL_RACE_ID not in by_race:
        msg = (
            f"{classified_cell.engine_fips}: pole {classified_cell.pole} PRESENT but county "
            "total (T, race_id=1) universe is missing"
        )
        raise ArtifactGenerationError(msg)
    total_u = by_race[TOTAL_RACE_ID].universe_u

    w = compute_w(below_b, universe_u, p_bar)
    damped = compute_damped_sigma(w, universe_u)
    mass_vs_settler_norm = below_b - universe_u * p_bar
    mass_vs_demonstrated_floor = universe_u * q_bar - below_b

    return CountyPoleRow(
        fips=classified_cell.engine_fips,
        pole=classified_cell.pole,
        pole_role=POLE_ROLE[classified_cell.pole],
        universe_u=universe_u,
        below_b=below_b,
        rate=classified_cell.rate,
        w=w,
        sigma_damped=damped.sigma_damped,
        damping_weight=damped.damping_weight,
        mass_vs_settler_norm=mass_vs_settler_norm,
        mass_vs_demonstrated_floor=mass_vs_demonstrated_floor,
        lambda_per_capita=mass_vs_settler_norm / total_u,
        omega_hat_per_capita=mass_vs_demonstrated_floor / total_u,
        absence_class="PRESENT",
        fips_source_vintage=classified_cell.fips_source_vintage,
    )


def build_county_pole_rows(
    universe_fips: frozenset[str],
    by_race_by_query_fips: Mapping[str, Mapping[int, PoleCellPair]],
    crosswalk_rows: Sequence[CrosswalkRow],
) -> tuple[CountyPoleRow, ...]:
    """Steps 3-8 assembled: A2's full row set for ``universe_fips`` — one
    row per ``(fips, pole)``, sorted ``(fips, pole)`` for determinism
    (brief step 2)."""
    classified = classify_universe_poles(universe_fips, by_race_by_query_fips, crosswalk_rows)
    pooled = compute_pooled_ratios(classified)
    rows = [
        _measure_row(
            c, by_race_by_query_fips.get(c.query_fips), p_bar=pooled.p_bar, q_bar=pooled.q_bar
        )
        for c in classified
    ]
    return tuple(sorted(rows, key=lambda r: (r.fips, r.pole)))


class PooledOverlap(NamedTuple):
    """G8's step-6 pooled disclosure — the raw pooled A/H/I sums plus the
    bound. Never subtracted from any pole sum anywhere in this module (the
    AST leg in ``test_national_incidence_guards_arithmetic.py`` scans the
    whole module source, including this section, for that violation)."""

    sum_total_a: int
    sum_white_non_hispanic_h: int
    sum_hispanic_i: int
    overlap_bound: int


def compute_pooled_overlap(
    universe_fips: frozenset[str],
    by_race_by_query_fips: Mapping[str, Mapping[int, PoleCellPair]],
    crosswalk_by_engine: Mapping[str, CrosswalkRow],
) -> PooledOverlap:
    """Step 6: G8's pooled ``I - (A - H)`` overlap bound (F2), summed over
    every county in ``universe_fips`` with A/H/I data present under its
    A1-resolved query fips. Disclosed only.
    """
    sum_a = sum_h = sum_i = 0
    for engine_fips in universe_fips:
        query_fips = resolve_query_fips(engine_fips, crosswalk_by_engine)
        by_race = by_race_by_query_fips.get(query_fips)
        if by_race is None:
            continue
        a_cell = by_race.get(WHITE_ALONE_RACE_ID)
        h_cell = by_race.get(POLE_RACE_ID[SETTLER_POLE_LETTER])
        i_cell = by_race.get(POLE_RACE_ID["I"])
        if a_cell is None or h_cell is None or i_cell is None:
            continue
        sum_a += a_cell.universe_u
        sum_h += h_cell.universe_u
        sum_i += i_cell.universe_u
    overlap_bound = overlap_upper_bound(sum_a, sum_h, sum_i)
    return PooledOverlap(
        sum_total_a=sum_a,
        sum_white_non_hispanic_h=sum_h,
        sum_hispanic_i=sum_i,
        overlap_bound=overlap_bound,
    )


class FloorAggregateRow(NamedTuple):
    """A3's row shape — one ``(pole, universe_variant)`` aggregate, plus a
    ``pole=POOLED_POLE_LABEL`` row per variant carrying F1's B+C+I figures
    and F2's overlap bound (individual pole rows leave those three columns
    ``None`` — the ratio and the overlap bound are cross-pole quantities,
    not single-pole ones)."""

    pole: str
    universe_variant: str
    counties_present: int
    counties_absent: int
    sum_u: int
    sum_b: int
    rate: float | None
    p_bar: float
    q_bar: float
    sum_mass_vs_settler_norm: float | None
    sum_mass_vs_demonstrated_floor: float | None
    ratio_bribe_to_deprivation: float | None
    overlap_upper_bound: int | None
    overlap_bound_share: float | None
    vintage_time_id: int
    notes: str


def _pole_aggregate_row(
    pole: str,
    variant_label: str,
    classified: Sequence[ClassifiedCell],
    universe_fips: frozenset[str],
    pooled_ratios: PooledRatios,
) -> FloorAggregateRow:
    """One individual-pole A3 row: G5's reconciliation, this pole's OWN
    ratio-of-sums rate (F5's ``q_pole`` — distinct from the pooled ``q̄``),
    and the two mass sums computed against the GLOBAL pooled p̄/q̄."""
    pole_cells = [c for c in classified if c.pole == pole]
    present = [c for c in pole_cells if c.absence_class == "PRESENT"]
    reconciliation = reconcile_absence_counts(
        [c.absence_class for c in pole_cells], universe_size=len(universe_fips)
    )
    sum_u = sum(_require_int(c.universe_u, context=f"{pole} present cell") for c in present)
    sum_b = sum(_require_int(c.below_b, context=f"{pole} present cell") for c in present)
    # G1 (ratio_of_sums), never a hand-inlined Σb/Σu — this row's own rate is
    # exactly the pooled-ratio LAW applied to a single pole's present cells,
    # so it goes through the same guarded function every other pooled rate does.
    present_agg_cells = [
        AggregationCell(
            fips=c.engine_fips,
            universe_u=_require_int(c.universe_u, context=f"{pole} present cell"),
            below_b=_require_int(c.below_b, context=f"{pole} present cell"),
        )
        for c in present
    ]
    own_rate = ratio_of_sums(present_agg_cells) if present else None
    sum_mvsn = (
        sum(
            _require_int(c.below_b, context=pole)
            - _require_int(c.universe_u, context=pole) * pooled_ratios.p_bar
            for c in present
        )
        if present
        else None
    )
    sum_mvdf = (
        sum(
            _require_int(c.universe_u, context=pole) * pooled_ratios.q_bar
            - _require_int(c.below_b, context=pole)
            for c in present
        )
        if present
        else None
    )
    return FloorAggregateRow(
        pole=pole,
        universe_variant=variant_label,
        counties_present=reconciliation.counties_present,
        counties_absent=reconciliation.counties_absent,
        sum_u=sum_u,
        sum_b=sum_b,
        rate=own_rate,
        p_bar=pooled_ratios.p_bar,
        q_bar=pooled_ratios.q_bar,
        sum_mass_vs_settler_norm=sum_mvsn,
        sum_mass_vs_demonstrated_floor=sum_mvdf,
        ratio_bribe_to_deprivation=None,
        overlap_upper_bound=None,
        overlap_bound_share=None,
        vintage_time_id=TIME_ID,
        notes=(
            f"individual pole row ({pole}); rate is this pole's own ratio-of-sums (F5's "
            "q_pole), not the pooled B+C+I q̄. ratio_bribe_to_deprivation and the overlap "
            f"bound are cross-pole quantities -- see the {POOLED_POLE_LABEL} row (F1/F2)."
        ),
    )


def _pooled_aggregate_row(
    variant_label: str,
    classified: Sequence[ClassifiedCell],
    universe_fips: frozenset[str],
    pooled_ratios: PooledRatios,
    by_race_by_query_fips: Mapping[str, Mapping[int, PoleCellPair]],
    crosswalk_by_engine: Mapping[str, CrosswalkRow],
) -> FloorAggregateRow:
    """A3's B+C+I POOLED row: F1's ΣE (oppressed deprivation mass) / ΣΩ
    (settler bribe mass) ratio, plus F2's overlap bound + share (against
    the B+C+I universe, ``Σu_o``)."""
    oppressed_cells = [c for c in classified if c.pole in OPPRESSED_POLE_LETTERS]
    present_oppressed = [c for c in oppressed_cells if c.absence_class == "PRESENT"]
    reconciliation = reconcile_absence_counts(
        [c.absence_class for c in oppressed_cells], universe_size=3 * len(universe_fips)
    )
    sum_u = sum(_require_int(c.universe_u, context="pooled") for c in present_oppressed)
    sum_b = sum(_require_int(c.below_b, context="pooled") for c in present_oppressed)
    sum_e = sum(
        _require_int(c.below_b, context="pooled")
        - _require_int(c.universe_u, context="pooled") * pooled_ratios.p_bar
        for c in present_oppressed
    )
    settler_present = [
        c for c in classified if c.pole == SETTLER_POLE_LETTER and c.absence_class == "PRESENT"
    ]
    sum_omega = sum(
        _require_int(c.universe_u, context="pooled") * pooled_ratios.q_bar
        - _require_int(c.below_b, context="pooled")
        for c in settler_present
    )
    ratio_bribe_to_deprivation = (sum_omega / sum_e) if sum_e != 0 else None

    pooled_overlap = compute_pooled_overlap(
        universe_fips, by_race_by_query_fips, crosswalk_by_engine
    )
    overlap_bound = pooled_overlap.overlap_bound
    overlap_bound_share = (overlap_bound / sum_u) if sum_u else None

    return FloorAggregateRow(
        pole=POOLED_POLE_LABEL,
        universe_variant=variant_label,
        counties_present=reconciliation.counties_present,
        counties_absent=reconciliation.counties_absent,
        sum_u=sum_u,
        sum_b=sum_b,
        rate=pooled_ratios.q_bar,
        p_bar=pooled_ratios.p_bar,
        q_bar=pooled_ratios.q_bar,
        sum_mass_vs_settler_norm=sum_e,
        sum_mass_vs_demonstrated_floor=sum_omega,
        ratio_bribe_to_deprivation=ratio_bribe_to_deprivation,
        overlap_upper_bound=overlap_bound,
        overlap_bound_share=overlap_bound_share,
        vintage_time_id=TIME_ID,
        notes=(
            "B+C+I pooled (F1, the ruled partition, OQ1). ratio_bribe_to_deprivation = "
            "ΣΩ (H settler-pole bribe mass) / ΣE (B+C+I deprivation mass). "
            "overlap_upper_bound = I-(A-H), disclosed and never netted out of any pole sum "
            "(G8/F2); overlap_bound_share is against sum_u (Σu_o, the B+C+I universe)."
        ),
    )


def build_reproduction_floor_rows(
    variant_name: str,
    universe_fips: frozenset[str],
    by_race_by_query_fips: Mapping[str, Mapping[int, PoleCellPair]],
    crosswalk_rows: Sequence[CrosswalkRow],
) -> tuple[FloorAggregateRow, ...]:
    """A3's five rows for ONE ``universe_variant`` — the four individual
    poles (B, C, I, H) plus the B+C+I :data:`POOLED_POLE_LABEL` row.
    ``universe_variant``'s label carries the MEASURED county count
    (``f"{name}_{len(universe_fips)}"``), never a hand-typed number — the
    plan's literal ``scopes_3140`` string is verified-wrong (T1/T2: 3,156)."""
    classified = classify_universe_poles(universe_fips, by_race_by_query_fips, crosswalk_rows)
    pooled_ratios = compute_pooled_ratios(classified)
    crosswalk_by_engine = {row.fips_engine: row for row in crosswalk_rows}
    variant_label = f"{variant_name}_{len(universe_fips)}"

    rows = [
        _pole_aggregate_row(pole, variant_label, classified, universe_fips, pooled_ratios)
        for pole in POLE_LETTERS
    ]
    rows.append(
        _pooled_aggregate_row(
            variant_label,
            classified,
            universe_fips,
            pooled_ratios,
            by_race_by_query_fips,
            crosswalk_by_engine,
        )
    )
    return tuple(rows)


def _open_deterministic_gzip_text(path: Path, compresslevel: int) -> io.TextIOWrapper:
    """Open ``path`` for gzip text-mode writing with a pinned header
    ``MTIME``. Copied verbatim from
    ``tools/make_faf_bloc_tons_artifact.py::_open_deterministic_gzip_text``
    (which copies ``make_lodes_tri_county_artifact.py``) — the
    byte-identity precondition for the double-run sha gate."""
    binary = gzip.GzipFile(filename=str(path), mode="wb", compresslevel=compresslevel, mtime=0)
    return io.TextIOWrapper(binary, encoding="utf-8", newline="")


_GZIP_COMPRESSLEVEL = 9

A2_COLUMNS: tuple[str, ...] = (
    "fips",
    "pole",
    "pole_role",
    "universe_u",
    "below_b",
    "rate",
    "w",
    "sigma_damped",
    "damping_weight",
    "mass_vs_settler_norm",
    "mass_vs_demonstrated_floor",
    "lambda_per_capita",
    "omega_hat_per_capita",
    "absence_class",
    "fips_source_vintage",
)

A3_COLUMNS: tuple[str, ...] = (
    "pole",
    "universe_variant",
    "counties_present",
    "counties_absent",
    "sum_u",
    "sum_b",
    "rate",
    "p_bar",
    "q_bar",
    "sum_mass_vs_settler_norm",
    "sum_mass_vs_demonstrated_floor",
    "ratio_bribe_to_deprivation",
    "overlap_upper_bound",
    "overlap_bound_share",
    "vintage_time_id",
    "notes",
)


def _fmt_float(value: float | None) -> str:
    """Pinned float formatting (brief step 2) — empty string for an absent
    measure cell, never a fabricated ``0.0`` (G2's law extended to CSV
    serialization)."""
    return "" if value is None else _FLOAT_FMT.format(value)


def _fmt_int(value: int | None) -> str:
    return "" if value is None else str(value)


def write_county_pole_artifact(rows: Sequence[CountyPoleRow], out_path: Path) -> tuple[int, str]:
    """Step 10: emit A2 — gzip, pinned mtime=0, sorted ``(fips, pole)``."""
    out_path.parent.mkdir(parents=True, exist_ok=True)
    ordered = sorted(rows, key=lambda r: (r.fips, r.pole))
    with _open_deterministic_gzip_text(out_path, _GZIP_COMPRESSLEVEL) as handle:
        writer = csv.writer(handle, lineterminator="\n")
        writer.writerow(A2_COLUMNS)
        for row in ordered:
            writer.writerow(
                [
                    row.fips,
                    row.pole,
                    row.pole_role,
                    _fmt_int(row.universe_u),
                    _fmt_int(row.below_b),
                    _fmt_float(row.rate),
                    _fmt_float(row.w),
                    _fmt_float(row.sigma_damped),
                    _fmt_float(row.damping_weight),
                    _fmt_float(row.mass_vs_settler_norm),
                    _fmt_float(row.mass_vs_demonstrated_floor),
                    _fmt_float(row.lambda_per_capita),
                    _fmt_float(row.omega_hat_per_capita),
                    row.absence_class,
                    row.fips_source_vintage,
                ]
            )
    return len(ordered), _sha256_file(out_path)


def write_reproduction_floor_artifact(
    rows: Sequence[FloorAggregateRow], out_path: Path
) -> tuple[int, str]:
    """Step 10: emit A3 — plain csv (no gzip; plan §2's A3 path has no
    ``.gz`` suffix), sorted ``(pole, universe_variant)``."""
    out_path.parent.mkdir(parents=True, exist_ok=True)
    ordered = sorted(rows, key=lambda r: (r.pole, r.universe_variant))
    with out_path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle, lineterminator="\n")
        writer.writerow(A3_COLUMNS)
        for row in ordered:
            writer.writerow(
                [
                    row.pole,
                    row.universe_variant,
                    row.counties_present,
                    row.counties_absent,
                    row.sum_u,
                    row.sum_b,
                    _fmt_float(row.rate),
                    _fmt_float(row.p_bar),
                    _fmt_float(row.q_bar),
                    _fmt_float(row.sum_mass_vs_settler_norm),
                    _fmt_float(row.sum_mass_vs_demonstrated_floor),
                    _fmt_float(row.ratio_bribe_to_deprivation),
                    _fmt_int(row.overlap_upper_bound),
                    _fmt_float(row.overlap_bound_share),
                    row.vintage_time_id,
                    row.notes,
                ]
            )
    return len(ordered), _sha256_file(out_path)


def build_arg_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--export-source",
        action="store_true",
        help=(
            "materialize the four registered parquet sources "
            f"({', '.join(SOURCE_TABLES)}) from the read-only reference DB via "
            "make_data_artifacts.export_table_parquet — the ONLY sanctioned "
            "parquet-writing path. PRODUCES the registered sources; run inside "
            "`mise exec -- uv run --frozen python ...` (pinned SQLite required). There is no "
            "--from-sqlite derivation flag — every other mode reads only the "
            "sha-pinned parquet sources."
        ),
    )
    parser.add_argument("--sqlite-path", type=Path, default=DEFAULT_SQLITE_PATH)
    parser.add_argument("--dist-dir", type=Path, default=DIST_DIR)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_arg_parser().parse_args(argv)

    if args.export_source:
        export_source_tables(args.sqlite_path, dist_dir=args.dist_dir)
        return 0

    verified = verify_source_provenance(args.dist_dir)
    print(f"[national-incidence] provenance verified (sha-pinned): {sorted(verified)}")

    cells = read_filtered_poverty_cells(verified["fact_census_poverty"], verified["dim_county"])
    print(
        f"[national-incidence] filtered cells pulled: {len(cells)} "
        f"(time_id={TIME_ID}, category_id in {CATEGORY_IDS}, race_id in {RACE_IDS})"
    )

    variants = resolve_universe_variants(
        sqlite_path=args.sqlite_path,
        poverty_parquet=verified["fact_census_poverty"],
        county_parquet=verified["dim_county"],
    )
    for variant in variants:
        print(f"[national-incidence] universe '{variant.name}': {len(variant.fips)} counties")

    crosswalk_rows = load_crosswalk()
    by_race_by_query_fips = build_county_race_totals(cells)

    artifact_variant = next(v for v in variants if v.name == "artifact")
    a2_rows = build_county_pole_rows(artifact_variant.fips, by_race_by_query_fips, crosswalk_rows)
    a2_row_count, a2_sha = write_county_pole_artifact(a2_rows, A2_OUTPUT_PATH)
    print(
        f"[national-incidence] A2 written: {a2_row_count} rows -> {A2_OUTPUT_PATH} "
        f"(sha256={a2_sha})"
    )

    a3_rows: list[FloorAggregateRow] = []
    for variant in variants:
        a3_rows.extend(
            build_reproduction_floor_rows(
                variant.name, variant.fips, by_race_by_query_fips, crosswalk_rows
            )
        )
    a3_row_count, a3_sha = write_reproduction_floor_artifact(a3_rows, A3_OUTPUT_PATH)
    print(
        f"[national-incidence] A3 written: {a3_row_count} rows -> {A3_OUTPUT_PATH} "
        f"(sha256={a3_sha})"
    )

    print(
        "\n[national-incidence] data-artifacts.yaml entries (paste manually -- T5's job; "
        "never hand-type a sha256):"
    )
    print("  national_incidence_county_pole:")
    print(f"    rows: {a2_row_count}")
    print(f"    sha256: {a2_sha}")
    print(f"    home: {A2_OUTPUT_PATH.relative_to(_REPO_ROOT).as_posix()}")
    print("  national_reproduction_floor:")
    print(f"    rows: {a3_row_count}")
    print(f"    sha256: {a3_sha}")
    print(f"    home: {A3_OUTPUT_PATH.relative_to(_REPO_ROOT).as_posix()}")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except ArtifactGenerationError as error:
        print(f"[national-incidence] ABORT: {error}", file=sys.stderr)
        sys.exit(2)
