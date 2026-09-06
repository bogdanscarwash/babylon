"""Generate the deterministic USScenario county-seed artifact.

This data-periphery tool reads the reference SQLite database and writes
``src/babylon/data/game/us_county_territories.json``. Reference-scope and
incidence-artifact builders consume its FIPS-sorted county universe. The
artifact contains reference data; it does not instantiate game state.

**Scope**: the canonical Python data-periphery county universe
(:func:`babylon.data.reference_scope._load_national_fips` --
state fips < '60', excludes the synthetic ``{state}999`` rest-of-state
placeholders), FIPS-sorted.

**Population policy**: reuse ``ReferenceDataCache``'s Census household-count
SUM, with QCEW employment SUM x 0.33 as its fallback. The artifact preserves
this historical estimate and identifies missing counties in ``gaps``.
The default year is 2010. These estimates are not an authored campaign's
population parameters or a runtime population authority.

**Centroid policy**: ``dim_county_geometry.centroid_lat``/``centroid_lon``,
real TIGER centroids (NOT the fabricated hex-cell centroids the old h3 grid
used). A small number of scoped counties lack a geometry row (a different,
non-overlapping set from the population gap) -- also recorded in ``gaps``.

**Retired-FIPS dedup (2026-07-19, scout finding; extended 2026-07-20, T4
review finding I-1)**: ``dim_county`` carries BOTH a retired FIPS and its
modern successor(s) as separate rows for the SAME physical county/area, in
two distinct forms:

- **Rename** (:data:`_RETIRED_FIPS_SUPERSEDED`, one successor each):
  ``46113`` (Shannon County, SD) -> ``46102`` (Oglala Lakota County, SD --
  2015 Census Bureau rename); ``02270`` (Wade Hampton Census Area, AK) ->
  ``02158`` (Kusilvak Census Area, AK -- 2015 Census Bureau rename, the exact
  same class of duplicate as Shannon/Oglala Lakota).
- **Split** (:data:`_RETIRED_FIPS_SPLIT`, two successors -- a rename map
  cannot express two successors): ``02261`` (Valdez-Cordova Census Area, AK)
  dissolved in 2019 into BOTH ``02063`` (Chugach Census Area) and ``02066``
  (Copper River Census Area) per Census Bureau change notice; the retired
  whole plus both halves are all present in ``dim_county``.

All pass ``_load_national_fips``'s scoping filter, so the raw national scope
double- (rename) or triple- (split) counts these areas. :func:`_dedup_retired_fips`
drops a retired key ONLY when ALL of its successor(s) are present in the same
extraction -- recorded in the artifact's ``exclusions`` list, never silently
dropped. A generator self-check asserts NO key from the union of both maps
survives dedup, failing loud (Constitution III.11) rather than shipping a
duplicated artifact.

These are declared, cited maps, NOT generic rename/split detection: if a
similar duplicate surfaces later, a map is extended with its own citation,
not inferred. Generic inference is deliberately out of scope -- e.g.
``51515``/``51019`` (Bedford city / Bedford County, VA) are correctly NOT
deduped: both were legitimately separate entities at the 2010 reference year
(Bedford city didn't merge into Bedford County until 2013), so a name- or
geography-based heuristic would wrongly collapse two real counties into one.
The raw reference-scope universe carries the same latent double/triple count.
That data-periphery concern remains outside this artifact's declared
deduplication rule.

Only reference-derived fields are baked into the artifact: ``fips``,
``county_name``, ``state_abbrev``, ``centroid_lat``/``lon``, ``population``,
and ``raw_material_value_millions``. The retired Python engine's scenario
classification is preserved in Git history.

**Raw-material value policy (2026-07-20, #39 T6, schema_version 2)**:
``fact_state_minerals.value_millions`` (USGS Mineral Commodity Summaries,
Program 22 Wave 1, 50 states) allocated state -> county
by land-area share: ``county_share = area_sq_km / Σ(area_sq_km over the
state's scoped counties with a geometry row)``. This is a declared derived allocation, not an observation of county mining.
A county's ``raw_material_value_millions`` is ``None`` (recorded in
``gaps``) when its state has no ``fact_state_minerals`` row (DC, PR -- USGS
covers the 50 states only) or the county itself has no
``dim_county_geometry`` row (no area to allocate a share from) -- never a
fabricated default.

Regeneration::

    uv run python tools/generate_us_county_territories.py               # year 2010
    uv run python tools/generate_us_county_territories.py --year 2019

The output is deterministic: counties are FIPS-sorted and every query is a
GROUP BY / ORDER BY over stable keys, so identical DB inputs yield an
identical artifact (identical ``content_hash``). Commit the regenerated
JSON and note the new hash.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

from babylon.data.reference_data_cache import ReferenceDataCache
from babylon.data.reference_scope import DEFAULT_SQLITE_PATH, _load_national_fips
from babylon.reference.database import get_normalized_session_factory
from babylon.reference.schema import DimCounty, DimCountyGeometry, DimState, FactStateMinerals

#: Reference year used by the committed county-universe artifact.
DEFAULT_POPULATION_YEAR = 2010

#: Retired FIPS -> modern successor FIPS for the SAME physical county, both
#: of which are present as separate `dim_county` rows (module docstring,
#: "Retired-FIPS dedup"). Verified empirically:
#: 46113 = Shannon County SD (retired); 46102 = Oglala Lakota County SD
#: (renamed 2015 per Census Bureau change notice) -- the modern successor
#: (scout finding, 2026-07-19). 02270 = Wade Hampton Census Area AK (retired);
#: 02158 = Kusilvak Census Area AK (renamed 2015 per Census Bureau change
#: notice) -- the modern successor (T4 review finding I-1, 2026-07-20).
#: A declared, cited exclusion map ONLY -- no generic rename inference; if
#: another duplicate surfaces later, extend this map with its own citation.
_RETIRED_FIPS_SUPERSEDED: dict[str, str] = {
    "46113": "46102",  # Shannon County, SD -> Oglala Lakota County, SD (2015 rename)
    "02270": "02158",  # Wade Hampton Census Area, AK -> Kusilvak Census Area, AK (2015 rename)
}

#: Retired FIPS -> (successor_a, successor_b) for a county SPLIT into TWO
#: modern successors -- a rename map (above) cannot express two successors,
#: hence this separate declared structure (module docstring, "Retired-FIPS
#: dedup"). Verified empirically (T4 review finding I-1, 2026-07-20): 02261 =
#: Valdez-Cordova Census Area AK (retired, dissolved 2019 per Census Bureau
#: change notice) into 02063 = Chugach Census Area AK and 02066 = Copper
#: River Census Area AK -- both halves. A declared, cited exclusion map
#: ONLY -- no generic split inference; if another split surfaces later,
#: extend this map with its own citation.
_RETIRED_FIPS_SPLIT: dict[str, tuple[str, str]] = {
    "02261": ("02063", "02066"),
}

_ARTIFACT_PATH = (
    Path(__file__).resolve().parent.parent
    / "src"
    / "babylon"
    / "data"
    / "game"
    / "us_county_territories.json"
)


def _load_county_geo_rows(
    session_factory: Any,
) -> dict[str, tuple[str, str, float | None, float | None, float | None]]:
    """Real per-county name/state/centroid/area rows, FIPS-keyed.

    Returns ``{fips: (county_name, state_abbrev, centroid_lat, centroid_lon,
    area_sq_km)}``. ``centroid_lat``/``centroid_lon``/``area_sq_km`` are
    ``None`` for counties without a ``dim_county_geometry`` row (empirically
    the same counties lack both -- ``area_sq_km`` is never null on a county
    that HAS a geometry row, verified against the reference DB at T6
    generation time).
    """
    with session_factory() as session:
        rows = (
            session.query(
                DimCounty.fips,
                DimCounty.county_name,
                DimState.state_abbrev,
                DimCountyGeometry.centroid_lat,
                DimCountyGeometry.centroid_lon,
                DimCountyGeometry.area_sq_km,
            )
            .join(DimState, DimState.state_id == DimCounty.state_id)
            .outerjoin(DimCountyGeometry, DimCountyGeometry.county_id == DimCounty.county_id)
            .all()
        )
    return {
        fips: (
            county_name,
            state_abbrev,
            float(lat) if lat is not None else None,
            float(lon) if lon is not None else None,
            float(area) if area is not None else None,
        )
        for fips, county_name, state_abbrev, lat, lon, area in rows
    }


def _load_state_minerals(session_factory: Any) -> dict[str, float]:
    """``{state_fips: value_millions}`` from ``fact_state_minerals`` (Program 22
    Wave 1) joined to ``dim_state`` -- 50 states, no DC/PR/territories (USGS
    Mineral Commodity Summaries covers the 50 states only)."""
    with session_factory() as session:
        rows = (
            session.query(DimState.state_fips, FactStateMinerals.value_millions)
            .join(FactStateMinerals, FactStateMinerals.state_id == DimState.state_id)
            .all()
        )
    return {
        state_fips: float(value_millions)
        for state_fips, value_millions in rows
        if value_millions is not None
    }


def _content_hash(counties: list[dict[str, Any]]) -> str:
    """SHA-256 over the canonical (sorted-key, compact) county-rows payload."""
    canonical = json.dumps(counties, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def _dedup_retired_fips(scope_fips: frozenset[str]) -> tuple[frozenset[str], list[dict[str, Any]]]:
    """Drop retired FIPS whose modern successor(s) are present in the same scope.

    Two declared, cited structures (module docstring, "Retired-FIPS dedup"):
    a rename map (:data:`_RETIRED_FIPS_SUPERSEDED`, one successor) and a
    split map (:data:`_RETIRED_FIPS_SPLIT`, two successors). A retired key is
    dropped ONLY when ALL of its successor(s) are ALSO present -- if any
    successor is somehow absent, the retired row is kept rather than
    silently losing the county/area entirely.

    Returns:
        ``(deduped_scope, exclusions)`` where ``exclusions`` names each
        dropped FIPS + its successor(s) + a citation, for the artifact's
        ``exclusions`` list. Rename entries carry ``successor_fips`` as a
        single string; split entries carry it as a two-element list
        (distinguished by the ``kind`` field: ``"rename"`` vs ``"split"``).
    """
    exclusions: list[dict[str, Any]] = []
    dropped: set[str] = set()

    for retired, successor in _RETIRED_FIPS_SUPERSEDED.items():
        if retired in scope_fips and successor in scope_fips:
            dropped.add(retired)
            exclusions.append(
                {
                    "fips": retired,
                    "kind": "rename",
                    "successor_fips": successor,
                    "reason": (
                        f"retired FIPS superseded by {successor} (same physical county, "
                        "renamed per Census Bureau change notice); both rows present in "
                        "dim_county -- dropping the retired one to prevent double-counting"
                    ),
                }
            )

    for retired, successors in _RETIRED_FIPS_SPLIT.items():
        if retired in scope_fips and all(s in scope_fips for s in successors):
            dropped.add(retired)
            exclusions.append(
                {
                    "fips": retired,
                    "kind": "split",
                    "successor_fips": list(successors),
                    "reason": (
                        f"retired FIPS split into {successors[0]} and {successors[1]} "
                        "(same physical area, dissolved per Census Bureau change notice); "
                        "the retired whole plus both successor rows are present in "
                        "dim_county -- dropping the retired whole to prevent triple-counting"
                    ),
                }
            )

    return frozenset(scope_fips - dropped), exclusions


def _allocate_raw_material_values(
    scope_fips: frozenset[str],
    areas: dict[str, float | None],
    state_minerals: dict[str, float],
) -> tuple[dict[str, float | None], dict[str, str]]:
    """State ``fact_state_minerals.value_millions`` allocated to counties by
    land-area share (module docstring, "Raw-material value policy").

    Args:
        scope_fips: Counties in scope (post retired-FIPS dedup).
        areas: ``{fips: area_sq_km}`` (``None`` = no ``dim_county_geometry`` row).
        state_minerals: ``{state_fips: value_millions}`` (50 states only).

    Returns:
        ``(values, gap_reasons)`` -- ``{fips: raw_material_value_millions}``
        (``None`` when unseedable) and ``{fips: reason}`` for exactly the
        ``None`` entries, for the caller to fold into the artifact's ``gaps``
        list alongside population/centroid (module docstring format).
    """
    state_area_totals: dict[str, float] = {}
    for fips in scope_fips:
        area = areas.get(fips)
        if area is not None:
            state_fips = fips[:2]
            state_area_totals[state_fips] = state_area_totals.get(state_fips, 0.0) + area

    values: dict[str, float | None] = {}
    gap_reasons: dict[str, str] = {}
    for fips in sorted(scope_fips):
        state_fips = fips[:2]
        state_value = state_minerals.get(state_fips)
        area = areas.get(fips)
        if state_value is None:
            values[fips] = None
            gap_reasons[fips] = f"no fact_state_minerals row for state={state_fips}"
        elif area is None:
            values[fips] = None
            gap_reasons[fips] = (
                f"no dim_county_geometry row for county={fips} "
                "(no area to allocate a state-mineral-value share from)"
            )
        else:
            values[fips] = state_value * (area / state_area_totals[state_fips])
    return values, gap_reasons


def build_payload(
    year: int = DEFAULT_POPULATION_YEAR,
    sqlite_path: Path = DEFAULT_SQLITE_PATH,
) -> dict[str, Any]:
    """Build the full county-seed artifact payload from the reference DB."""
    raw_scope_fips = _load_national_fips(sqlite_path)
    scope_fips, exclusions = _dedup_retired_fips(raw_scope_fips)
    session_factory = get_normalized_session_factory()
    geo_rows = _load_county_geo_rows(session_factory)
    state_minerals = _load_state_minerals(session_factory)

    cache = ReferenceDataCache(sqlite_path)
    cache.hydrate(scope_fips=scope_fips, year_set=frozenset({year}))

    areas = {fips: geo_rows[fips][4] for fips in scope_fips if fips in geo_rows}
    raw_material_values, raw_material_gap_reasons = _allocate_raw_material_values(
        scope_fips, areas, state_minerals
    )

    counties: list[dict[str, Any]] = []
    gaps: list[dict[str, str]] = []
    for fips in sorted(scope_fips):
        county_name, state_abbrev, lat, lon, _area = geo_rows.get(
            fips, (fips, "", None, None, None)
        )

        population = cache.lookup_population(fips, year)
        if population is None:
            gaps.append(
                {
                    "fips": fips,
                    "field": "population",
                    "reason": f"no Census or QCEW row for county={fips} year={year}",
                }
            )

        if lat is None or lon is None:
            gaps.append(
                {
                    "fips": fips,
                    "field": "centroid",
                    "reason": f"no dim_county_geometry row for county={fips}",
                }
            )

        raw_material_reason = raw_material_gap_reasons.get(fips)
        if raw_material_reason is not None:
            gaps.append(
                {
                    "fips": fips,
                    "field": "raw_material_value_millions",
                    "reason": raw_material_reason,
                }
            )

        counties.append(
            {
                "fips": fips,
                "county_name": county_name,
                "state_abbrev": state_abbrev,
                "centroid_lat": lat,
                "centroid_lon": lon,
                "population": population,
                "raw_material_value_millions": raw_material_values.get(fips),
            }
        )

    # Self-check (2026-07-19 scout finding; extended 2026-07-20, T4 review
    # finding I-1): no retired FIPS from the UNION of both exclusion maps
    # (renames + splits) may survive dedup -- a survivor here means the drop
    # logic failed silently, which must fail LOUD (Constitution III.11)
    # rather than ship a duplicated county/area.
    final_fips = {c["fips"] for c in counties}
    declared_exclusion_keys = set(_RETIRED_FIPS_SUPERSEDED) | set(_RETIRED_FIPS_SPLIT)
    survivors = final_fips & declared_exclusion_keys
    if survivors:
        raise AssertionError(
            f"retired FIPS {sorted(survivors)} survived _dedup_retired_fips -- "
            "dedup logic is broken, refusing to write a duplicated artifact"
        )

    return {
        "schema_version": 2,
        "source": {
            "dataset": (
                "dim_county + dim_county_geometry + fact_census_income (Census) "
                "with fact_qcew_annual (QCEW) fallback + fact_state_minerals "
                "(USGS MCS, Program 22 Wave 1)"
            ),
            "reference_db": "data/sqlite/marxist-data-3NF.sqlite",
            "scope_rule": (
                "national (mirrors data.reference_scope._load_national_fips: "
                "state fips < '60', excludes {state}999 placeholders) minus the "
                "retired-FIPS dedup (see 'exclusions')"
            ),
            "population_year": year,
            "population_policy": (
                "Census fact_census_income household_count SUM primary; QCEW "
                "fact_qcew_annual employment SUM x 0.33 fallback (matches "
                "ReferenceDataCache._resolve_population byte-for-byte -- reused "
                "directly, not reimplemented)"
            ),
            "raw_material_value_policy": (
                "fact_state_minerals.value_millions (50 states) allocated to "
                "counties by dim_county_geometry.area_sq_km share within the "
                "state (schema_version 2, #39 T6) -- None when the state has "
                "no fact_state_minerals row (DC/PR) or the county has no "
                "geometry row (see 'gaps')"
            ),
            "county_count": len(counties),
        },
        "content_hash": _content_hash(counties),
        "counties": counties,
        "gaps": gaps,
        "exclusions": exclusions,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--year",
        type=int,
        default=DEFAULT_POPULATION_YEAR,
        help=f"Census/QCEW year to resolve population from (default: {DEFAULT_POPULATION_YEAR})",
    )
    args = parser.parse_args()

    payload = build_payload(args.year)
    _ARTIFACT_PATH.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {_ARTIFACT_PATH}")
    print(
        f"  counties={payload['source']['county_count']}  year={payload['source']['population_year']}"
        f"  content_hash={payload['content_hash'][:16]}..."
    )
    print(f"  exclusions={len(payload['exclusions'])}")
    for excl in payload["exclusions"]:
        successor = excl["successor_fips"]
        successor_display = "+".join(successor) if isinstance(successor, list) else successor
        print(f"    {excl['fips']} -> {successor_display} [{excl['kind']}]: {excl['reason']}")
    print(f"  gaps={len(payload['gaps'])}")
    for gap in payload["gaps"]:
        print(f"    {gap['fips']} [{gap['field']}]: {gap['reason']}")


if __name__ == "__main__":
    main()
