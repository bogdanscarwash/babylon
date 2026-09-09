#!/usr/bin/env python3
"""FIPS vintage crosswalk artifact generator (#334 Phase 0, A1 + guard G7).

Reconciles two independent county universes and the ACS-2019 (``time_id=23``,
``fact_census_poverty``) data-availability gaps between them, then emits a
small, hand-authored, checked-in crosswalk:

- **The engine universe** — ``src/babylon/data/game/us_county_territories.json``
  (3,153 counties, current TIGER-vintage FIPS).
- **The resolver universe** — ``babylon.data.reference_scope.
  _load_national_fips`` against ``dim_county`` (3,156 counties; admits three
  retired FIPS the engine artifact does not: ``02261`` Valdez-Cordova,
  ``02270`` Wade Hampton, ``46113`` Shannon).

Every engine-universe county absent from ``fact_census_poverty`` at
``time_id=23`` (14 counties, verified against the read-only reference DB —
see "Deviation from the plan's framing" below for the verification queries'
findings) gets a crosswalk row, classifying it into exactly one of:

``DECLARED_HOLE``
    Zero ``fact_census_poverty`` rows under **any** FIPS code this county has
    ever held, at **any** ``time_id``. Never a valid crosswalk target (G7).
    Verified for ``46102`` Oglala Lakota SD (Pine Ridge; predecessor ``46113``
    carries rows only 2010-2014) and ``02158`` Kusilvak Census Area AK
    (predecessor ``02270`` Wade Hampton likewise carries rows only 2010-2014
    — the identical administrative pattern as Pine Ridge, verified this pass;
    see "Deviation from the plan's framing" below).
``SPLIT_UNRESOLVED``
    A single predecessor with full 2019 data was split into two-or-more
    current engine counties (``02063`` Chugach + ``02066`` Copper River, both
    ex ``02261`` Valdez-Cordova, effective 2019). The predecessor's 2019
    headcount is population for the *combined* pre-split territory and
    cannot be disaggregated between the successors without the deleted
    census extractor (``data-catalog.yaml:1096``) — attributing it wholesale
    to either successor would double-count that population if both rows
    fed a summed national total (the exact failure mode G7's injectivity
    check exists to catch), so both rows carry ``recoverable=false`` with
    the known-but-unusable source disclosed informationally.
``REORGANIZATION_UNRESOLVED``
    Connecticut's 2022 replacement of 8 counties with 9 planning regions
    (``09110``...``09190``) is a genuine many-to-many restructuring, not a
    set of 1:1 renames or clean splits — confirmed against the CT Data
    Collaborative / zip-codes.com correspondence: every planning region
    draws territory from 2-3 of the 8 former counties, and no county is the
    sole source of any single region. No single-FIPS predecessor can be
    assigned per region without fabricating precision the underlying
    geography does not support, so all 9 rows carry ``recoverable=false``.
``RENAME_RECOVERABLE``
    A clean, exclusive, unambiguous 1:1 predecessor with full 2019 data
    (``51515`` Bedford city VA, reverted to a town and annexed wholly into
    Bedford County ``51019`` in 2013 — verified: ``51019`` carries full 2019
    rows and no other county's territory is involved).
``RESOLVER_ONLY``
    A FIPS the resolver's ``dim_county``-based universe admits but the
    engine's territory list does not carry at all (``02261``, ``02270``,
    ``46113`` — the three-FIPS universe-size delta). These are documentation
    rows recording *why* the resolver (3,156) and engine (3,153) universe
    sizes differ by exactly 3, not ACS-recovery rows.

**Deviation from the plan's framing, verified this pass (2026-08-17) against
the read-only reference DB** (``data/sqlite/marxist-data-3NF.sqlite``,
``mode=ro``): the plan's proposal source
(``reports/national-oppression-proposal.md:127-136``) and the implementation
plan's T1 section characterize the 13 non-hole absences as uniformly
"recoverable" (grouping ``02063/02066/02158`` together as "3 AK
reorganizations" and all 9 CT regions as recoverable via "09001...09015").
Direct queries against ``fact_census_poverty`` show this does not hold for
**7 of those 13**:

- ``02158`` Kusilvak has **zero** rows at any ``time_id`` under either its own
  FIPS or its named predecessor ``02270`` (which stops at 2014, exactly
  mirroring Pine Ridge/``46113``) — reclassified ``DECLARED_HOLE``, not
  recoverable. G5 (T3b, downstream) already forbids imputing ``46102`` from
  ``46113``'s stale 2010-2014 rows; the identical logic forbids imputing
  ``02158`` from ``02270``'s equally-stale rows, so treating one as a hole
  and the other as recoverable would be an inconsistent standard.
- ``02063``/``02066`` share a single legitimate source (``02261``) that
  cannot be independently attributed to each successor without either
  fabricating a disaggregation or double-counting the shared population
  under G7's injectivity guard — reclassified ``SPLIT_UNRESOLVED``.
- The 9 CT planning regions have **no** 1:1 predecessor at all (verified: a
  genuine many-to-many correspondence, not merely an 8-vs-9 counting
  mismatch) — reclassified ``REORGANIZATION_UNRESOLVED``.

Only ``51515`` (Bedford city VA) survives as genuinely, unambiguously
``recoverable=true``. This does **not** change the row count (still 17: 14
engine-domain absence rows + 3 resolver-only delta rows) or any of F1-F7's
national-aggregate findings (all are dominated by large-population counties;
these are a handful of small Alaska/CT counties). It changes downstream
consumers' expectations: T4's derivation must **not** treat ``02063``,
``02066``, or ``02158`` as recovered-via-crosswalk. Flagged prominently in
``task-1-report.md`` for Director/reviewer visibility — this is a factual
correction against the plan's stated row classification, not a scope change.

This tool does NOT write ``data-artifacts.yaml`` (T5's job; ADR121
hand-maintained-artifact pattern, mirroring
``tools/make_faf_bloc_tons_artifact.py``) — it prints the manifest block for
hand-entry. ``tests/unit/tools/test_fips_vintage_crosswalk.py`` carries G7's
guard tests (``TestMutationG7``) plus the universe-enumeration checks.

Usage::

    uv run python tools/make_fips_vintage_crosswalk.py
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import sys
from pathlib import Path
from typing import NamedTuple

_REPO_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(_REPO_ROOT / "src"))

from babylon.data.reference_scope import (  # noqa: E402
    DEFAULT_SQLITE_PATH,
    _load_national_fips,
)

ENGINE_TERRITORIES_JSON = _REPO_ROOT / "src/babylon/data/game/us_county_territories.json"
DEFAULT_OUT = _REPO_ROOT / "src/babylon/data/reference/national/county_fips_vintage_crosswalk.csv"

#: Relation vocabulary. ``DECLARED_HOLE`` name is fixed by the plan (§2 A1,
#: reused verbatim by A2's ``absence_class`` enum, §2 A2) — the other four
#: are this module's own, chosen to make each row's recoverability reasoning
#: legible without cross-referencing prose.
RELATION_DECLARED_HOLE = "DECLARED_HOLE"
RELATION_SPLIT_UNRESOLVED = "SPLIT_UNRESOLVED"
RELATION_REORGANIZATION_UNRESOLVED = "REORGANIZATION_UNRESOLVED"
RELATION_RENAME_RECOVERABLE = "RENAME_RECOVERABLE"
RELATION_RESOLVER_ONLY = "RESOLVER_ONLY"

CROSSWALK_COLUMNS: tuple[str, ...] = (
    "fips_engine",
    "fips_acs2019",
    "relation",
    "vintage_note",
    "recoverable",
)


class CrosswalkRow(NamedTuple):
    """One A1 row. ``fips_acs2019=""`` means no usable/known 2019 ACS source
    (either a true hole or a documentation-only resolver-delta row).
    ``recoverable`` is true only when ``fips_acs2019`` is a safe, exclusive,
    non-double-counting source for ``fips_engine``."""

    fips_engine: str
    fips_acs2019: str
    relation: str
    vintage_note: str
    recoverable: bool


class ArtifactGenerationError(Exception):
    """A generation step failed loudly."""


class CrosswalkValidationError(ArtifactGenerationError):
    """G7: the crosswalk violates one of its three structural laws."""


#: The 17 hand-authored rows (proposal absence table,
#: ``reports/national-oppression-proposal.md:127-136``; verified against
#: ``data/sqlite/marxist-data-3NF.sqlite`` this pass — see module docstring
#: "Deviation from the plan's framing"). Sorted by ``fips_engine`` at
#: authoring time for reviewability; the emitter re-sorts defensively.
CROSSWALK_ROWS: tuple[CrosswalkRow, ...] = (
    CrosswalkRow(
        fips_engine="02063",
        fips_acs2019="02261",
        relation=RELATION_SPLIT_UNRESOLVED,
        vintage_note=(
            "Chugach Census Area AK, effective 2019 split of Valdez-Cordova Census "
            "Area (02261). 02261 carries full 2010-2019 fact_census_poverty rows "
            "(verified), but that data covers the combined pre-split territory and "
            "cannot be disaggregated between Chugach and Copper River (02066) "
            "without the deleted census extractor (data-catalog.yaml:1096) — "
            "attributing it to both successors would double-count the shared "
            "population under G7's injectivity guard, so this row is disclosed "
            "(informational fips_acs2019) but not recoverable."
        ),
        recoverable=False,
    ),
    CrosswalkRow(
        fips_engine="02066",
        fips_acs2019="02261",
        relation=RELATION_SPLIT_UNRESOLVED,
        vintage_note=(
            "Copper River Census Area AK, effective 2019 split of Valdez-Cordova "
            "Census Area (02261). See 02063's note — same shared, non-disaggregable "
            "source; not independently recoverable."
        ),
        recoverable=False,
    ),
    CrosswalkRow(
        fips_engine="02158",
        fips_acs2019="",
        relation=RELATION_DECLARED_HOLE,
        vintage_note=(
            "Kusilvak Census Area AK, renamed from Wade Hampton Census Area (02270) "
            "2015. Verified: zero fact_census_poverty rows at any time_id under "
            "02158, and 02270 carries rows only 2010-2014 (stops before the pinned "
            "2019 vintage) — the identical administrative pattern as Pine Ridge "
            "(46102/46113). Deviates from the plan's '13 recoverable' framing, "
            "which grouped this with the AK 02063/02066 split; reclassified here "
            "after direct DB verification (see module docstring). Never imputed "
            "from 02270's stale rows (G5's Pine-Ridge logic applies identically)."
        ),
        recoverable=False,
    ),
    CrosswalkRow(
        fips_engine="09110",
        fips_acs2019="",
        relation=RELATION_REORGANIZATION_UNRESOLVED,
        vintage_note=(
            "Capitol Planning Region CT, 2022 county-equivalent replacement. Draws "
            "territory from Hartford, Tolland, and Windham Counties (verified "
            "many-to-many correspondence, CT Data Collaborative crosswalk) — no "
            "single former county is this region's exclusive predecessor, so no "
            "fips_acs2019 target is assigned (fabricating one would misattribute "
            "population with false precision)."
        ),
        recoverable=False,
    ),
    CrosswalkRow(
        fips_engine="09120",
        fips_acs2019="",
        relation=RELATION_REORGANIZATION_UNRESOLVED,
        vintage_note=(
            "Greater Bridgeport Planning Region CT, 2022 county-equivalent "
            "replacement. Draws territory from Fairfield County plus adjoining "
            "planning-region splits (many-to-many; see 09110's note) — no exclusive "
            "predecessor."
        ),
        recoverable=False,
    ),
    CrosswalkRow(
        fips_engine="09130",
        fips_acs2019="",
        relation=RELATION_REORGANIZATION_UNRESOLVED,
        vintage_note=(
            "Lower Connecticut River Valley Planning Region CT, 2022 "
            "county-equivalent replacement. Draws territory from Middlesex and New "
            "London Counties (many-to-many; see 09110's note) — no exclusive "
            "predecessor."
        ),
        recoverable=False,
    ),
    CrosswalkRow(
        fips_engine="09140",
        fips_acs2019="",
        relation=RELATION_REORGANIZATION_UNRESOLVED,
        vintage_note=(
            "Naugatuck Valley Planning Region CT, 2022 county-equivalent "
            "replacement. Draws territory from Hartford, Litchfield, and New Haven "
            "Counties (many-to-many; see 09110's note) — no exclusive predecessor."
        ),
        recoverable=False,
    ),
    CrosswalkRow(
        fips_engine="09150",
        fips_acs2019="",
        relation=RELATION_REORGANIZATION_UNRESOLVED,
        vintage_note=(
            "Northeastern Connecticut Planning Region CT, 2022 county-equivalent "
            "replacement. Draws territory from Windham and New London Counties "
            "(many-to-many; see 09110's note) — no exclusive predecessor."
        ),
        recoverable=False,
    ),
    CrosswalkRow(
        fips_engine="09160",
        fips_acs2019="",
        relation=RELATION_REORGANIZATION_UNRESOLVED,
        vintage_note=(
            "Northwest Hills Planning Region CT, 2022 county-equivalent "
            "replacement. Draws territory from Hartford and Litchfield Counties "
            "(many-to-many; see 09110's note) — no exclusive predecessor."
        ),
        recoverable=False,
    ),
    CrosswalkRow(
        fips_engine="09170",
        fips_acs2019="",
        relation=RELATION_REORGANIZATION_UNRESOLVED,
        vintage_note=(
            "South Central Connecticut Planning Region CT, 2022 county-equivalent "
            "replacement. Draws territory from New Haven County plus adjoining "
            "planning-region splits (many-to-many; see 09110's note) — no exclusive "
            "predecessor."
        ),
        recoverable=False,
    ),
    CrosswalkRow(
        fips_engine="09180",
        fips_acs2019="",
        relation=RELATION_REORGANIZATION_UNRESOLVED,
        vintage_note=(
            "Southeastern Connecticut Planning Region CT, 2022 county-equivalent "
            "replacement. Draws territory from New London and Windham Counties "
            "(many-to-many; see 09110's note) — no exclusive predecessor."
        ),
        recoverable=False,
    ),
    CrosswalkRow(
        fips_engine="09190",
        fips_acs2019="",
        relation=RELATION_REORGANIZATION_UNRESOLVED,
        vintage_note=(
            "Western Connecticut Planning Region CT, 2022 county-equivalent "
            "replacement. Draws territory from Fairfield and Litchfield Counties "
            "(many-to-many; see 09110's note) — no exclusive predecessor."
        ),
        recoverable=False,
    ),
    CrosswalkRow(
        fips_engine="46102",
        fips_acs2019="",
        relation=RELATION_DECLARED_HOLE,
        vintage_note=(
            "Oglala Lakota County SD (Pine Ridge), renamed from Shannon County "
            "(46113) 2015. Verified: zero fact_census_poverty rows at any time_id "
            "under 46102, and 46113 carries rows only 2010-2014 (stops before the "
            "pinned 2019 vintage). Permanent declared hole until a source exists "
            "(proposal §9 standing cost); never imputed from 46113 (G5)."
        ),
        recoverable=False,
    ),
    CrosswalkRow(
        fips_engine="51515",
        fips_acs2019="51019",
        relation=RELATION_RENAME_RECOVERABLE,
        vintage_note=(
            "Bedford city VA reverted to town status and was annexed wholly into "
            "Bedford County (51019) effective 2013 — a clean, exclusive merge (no "
            "other county's territory involved). Verified: 51019 carries full 2019 "
            "fact_census_poverty rows."
        ),
        recoverable=True,
    ),
    CrosswalkRow(
        fips_engine="02261",
        fips_acs2019="",
        relation=RELATION_RESOLVER_ONLY,
        vintage_note=(
            "Valdez-Cordova Census Area AK. Admitted by scopes.py's "
            "_load_national_fips resolver (present in dim_county, state<60, "
            "not *999) but absent from the engine's us_county_territories.json "
            "territory list (superseded there by its 2019 split successors 02063/"
            "02066). Carries full 2019 rows itself (it is 02063/02066's shared "
            "source, see those rows) — this row documents the resolver/engine "
            "universe-size delta, not an ACS-recovery gap."
        ),
        recoverable=False,
    ),
    CrosswalkRow(
        fips_engine="02270",
        fips_acs2019="",
        relation=RELATION_RESOLVER_ONLY,
        vintage_note=(
            "Wade Hampton Census Area AK. Admitted by the resolver's dim_county "
            "universe but absent from the engine's territory list (superseded by "
            "its 2015 rename successor 02158 Kusilvak). Carries fact_census_poverty "
            "rows only 2010-2014 (itself the reason 02158 is DECLARED_HOLE, not "
            "recoverable) — documents the universe-size delta."
        ),
        recoverable=False,
    ),
    CrosswalkRow(
        fips_engine="46113",
        fips_acs2019="",
        relation=RELATION_RESOLVER_ONLY,
        vintage_note=(
            "Shannon County SD. Admitted by the resolver's dim_county universe but "
            "absent from the engine's territory list (superseded by its 2015 "
            "rename successor 46102 Oglala Lakota / Pine Ridge). Carries "
            "fact_census_poverty rows only 2010-2014 (itself the reason 46102 is "
            "DECLARED_HOLE) — documents the universe-size delta."
        ),
        recoverable=False,
    ),
)


def _enumerate_engine_universe(territories_path: Path) -> frozenset[str]:
    """The engine's territory FIPS set (``us_county_territories.json``)."""
    payload = json.loads(territories_path.read_text(encoding="utf-8"))
    return frozenset(county["fips"] for county in payload["counties"])


def _enumerate_resolver_universe(sqlite_path: Path) -> frozenset[str]:
    """The resolver's FIPS set, reusing ``scopes._load_national_fips``
    directly (DRY — same query the headless runner's ``--scope=national``
    resolves against)."""
    return _load_national_fips(sqlite_path)


def validate_crosswalk(rows: tuple[CrosswalkRow, ...]) -> None:
    """G7: the three structural laws.

    1. **Partial function** — ``fips_engine`` is a unique key; A1 has at most
       one row per engine FIPS.
    2. **Injective on fips_acs2019** — scoped to ``recoverable=True`` rows
       (the only rows T4 actually reads a source value from): no two such
       rows may share a target, which would double-count that target's
       population if both fed a summed national total. Non-recoverable rows
       may disclose a known-but-unusable shared source informationally
       (``SPLIT_UNRESOLVED``) without tripping this check — they are never
       consumed as a data source.
    3. **DECLARED_HOLE is never a target** — no row's ``fips_acs2019`` may
       equal the ``fips_engine`` of a row whose ``relation`` is
       ``DECLARED_HOLE`` (a hole has no data to point to, by definition).

    :raises CrosswalkValidationError: on any violation, naming every
        offending row (never a bare pass/fail).
    """
    engine_fips_seen: dict[str, int] = {}
    for row in rows:
        engine_fips_seen[row.fips_engine] = engine_fips_seen.get(row.fips_engine, 0) + 1
    duplicated = sorted(fips for fips, n in engine_fips_seen.items() if n > 1)
    if duplicated:
        msg = f"G7 partial-function violation: fips_engine appears more than once for {duplicated}"
        raise CrosswalkValidationError(msg)

    target_owners: dict[str, list[str]] = {}
    for row in rows:
        if row.recoverable and row.fips_acs2019:
            target_owners.setdefault(row.fips_acs2019, []).append(row.fips_engine)
    colliding = {target: owners for target, owners in target_owners.items() if len(owners) > 1}
    if colliding:
        msg = (
            f"G7 injectivity violation: recoverable rows sharing a fips_acs2019 target: {colliding}"
        )
        raise CrosswalkValidationError(msg)

    declared_holes = {row.fips_engine for row in rows if row.relation == RELATION_DECLARED_HOLE}
    hole_targets = sorted(
        row.fips_engine for row in rows if row.fips_acs2019 and row.fips_acs2019 in declared_holes
    )
    if hole_targets:
        msg = (
            f"G7 DECLARED_HOLE-as-target violation: rows targeting a declared hole: {hole_targets}"
        )
        raise CrosswalkValidationError(msg)


def _sha256_bytes(data: bytes) -> str:
    digest = hashlib.sha256()
    digest.update(data)
    return digest.hexdigest()


def _render_csv(rows: tuple[CrosswalkRow, ...]) -> str:
    """Deterministic CSV text: sorted by ``fips_engine``, LF endings."""
    import io

    buffer = io.StringIO(newline="")
    writer = csv.writer(buffer, lineterminator="\n")
    writer.writerow(CROSSWALK_COLUMNS)
    for row in sorted(rows, key=lambda r: r.fips_engine):
        writer.writerow(
            [
                row.fips_engine,
                row.fips_acs2019,
                row.relation,
                row.vintage_note,
                "true" if row.recoverable else "false",
            ]
        )
    return buffer.getvalue()


def _write_artifact(out_path: Path, rows: tuple[CrosswalkRow, ...]) -> tuple[int, str]:
    text = _render_csv(rows)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(text, encoding="utf-8")
    return len(rows), _sha256_bytes(text.encode("utf-8"))


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--out",
        type=Path,
        default=DEFAULT_OUT,
        help="Checked-in output path (repo-relative).",
    )
    parser.add_argument(
        "--territories-json",
        type=Path,
        default=ENGINE_TERRITORIES_JSON,
        help="Engine territory artifact (repo-relative).",
    )
    parser.add_argument(
        "--sqlite-path",
        type=Path,
        default=DEFAULT_SQLITE_PATH,
        help="Read-only reference DB, for resolver-universe enumeration.",
    )
    args = parser.parse_args(argv)

    print("[fips-crosswalk] validating G7 (partial function, injectivity, no hole targets)...")
    validate_crosswalk(CROSSWALK_ROWS)
    print(f"[fips-crosswalk] G7 clean: {len(CROSSWALK_ROWS)} rows")

    engine_universe = _enumerate_engine_universe(args.territories_json)
    print(f"[fips-crosswalk] engine universe: {len(engine_universe)} counties")
    if args.sqlite_path.exists():
        resolver_universe = _enumerate_resolver_universe(args.sqlite_path)
        print(f"[fips-crosswalk] resolver universe: {len(resolver_universe)} counties")
        delta = resolver_universe - engine_universe
        print(f"[fips-crosswalk] resolver-only delta ({len(delta)}): {sorted(delta)}")
    else:
        print(
            f"[fips-crosswalk] resolver universe: SKIPPED ({args.sqlite_path} not found)",
            file=sys.stderr,
        )

    rows, sha = _write_artifact(args.out, CROSSWALK_ROWS)
    print(f"[fips-crosswalk] {rows} rows -> {args.out} (sha256={sha})")
    print(
        "\n[fips-crosswalk] data-artifacts.yaml entry (paste manually — T5's job, "
        "no sqlite table backs this):"
    )
    print("  county_fips_vintage_crosswalk:")
    print(f"    rows: {rows}")
    print(f"    sha256: {sha}")
    print(f"    home: {args.out.relative_to(_REPO_ROOT).as_posix()}")
    recoverable_fips = sorted(r.fips_engine for r in CROSSWALK_ROWS if r.recoverable)
    hole_fips = sorted(
        r.fips_engine for r in CROSSWALK_ROWS if r.relation == RELATION_DECLARED_HOLE
    )
    print(f"[fips-crosswalk] recoverable=true rows ({len(recoverable_fips)}): {recoverable_fips}")
    print(f"[fips-crosswalk] DECLARED_HOLE rows ({len(hole_fips)}): {hole_fips}")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except ArtifactGenerationError as error:
        print(f"[fips-crosswalk] ABORT: {error}", file=sys.stderr)
        sys.exit(2)
