"""Read-only SQLite reference-data cache for Python data tooling.

Spec: 069-sqlite-cache-optimization.

The spec-066-bridged headless runner's ``persist_tick`` step opens fresh
``sqlite3.Connection`` instances per ``(county, tick)`` for two
reference-data lookups (population, employment-proxy). The returned
values are invariant within a calendar year, so under the weekly
cadence (``year = start_year + tick // 52``) the same fetch is
re-executed 51 redundant times before the year rolls over.

The playable runtime no longer imports this module. Artifact builders use its
batched, deterministic Census and QCEW resolution rules so regeneration stays
byte-equivalent to the frozen reference behavior.

See Also:
    ``specs/069-sqlite-cache-optimization/spec.md`` — feature spec.
    ``specs/069-sqlite-cache-optimization/contracts/reference_data_cache_contract.md``
        — cache class contract.
    ``specs/069-sqlite-cache-optimization/contracts/instrumentation_contract.md``
        — bridge read-counter contract.
    ``specs/069-sqlite-cache-optimization/data-model.md`` — entity inventory.
    ``specs/069-sqlite-cache-optimization/research.md`` — design decisions
        (R1-R11).
    :mod:`babylon.persistence.county_aggregation` — the legacy fetchers
        whose values the cache memoizes; unchanged per spec-069 R10
        (II.11 subsystem ownership).
"""

from __future__ import annotations

import sqlite3
from pathlib import Path

from pydantic import BaseModel, ConfigDict, Field

__all__ = [
    "ReferenceCacheEntry",
    "ReferenceDataCache",
    "derive_year_set",
]


# Wayne-calibrated employment-to-population ratio used by the legacy
# ``fetch_population_for_county_at_tick`` fallback. Preserved here for
# numeric equivalence (FR-005) with the pre-cache code path.
_QCEW_TO_POP_FALLBACK_RATIO = 0.33


def derive_year_set(start_year: int, total_ticks: int) -> frozenset[int]:
    """Enumerate the calendar years touched by a ``total_ticks``-tick run.

    Under the weekly cadence (``year = start_year + tick // 52``) the
    in-scope year set is mechanical:

        derive_year_set(start_year, total_ticks) =
            {start_year + i // 52 : i in [0, total_ticks - 1]}
            = {start_year, ..., start_year + (total_ticks - 1) // 52}

    Args:
        start_year:  Calendar year for tick 0.
        total_ticks: Number of ticks in the run.

    Returns:
        Frozen set of distinct calendar years touched. Empty for
        ``total_ticks <= 0`` (degenerate zero-tick run; spec-069
        Edge Cases).

    Example:
        >>> sorted(derive_year_set(2010, 520))
        [2010, 2011, 2012, 2013, 2014, 2015, 2016, 2017, 2018, 2019]
        >>> derive_year_set(2010, 0)
        frozenset()
        >>> sorted(derive_year_set(2010, 53))
        [2010, 2011]
    """
    if total_ticks <= 0:
        return frozenset()
    last_year = start_year + (total_ticks - 1) // 52
    return frozenset(range(start_year, last_year + 1))


class ReferenceCacheEntry(BaseModel):
    """One ``(county_fips, year)`` tuple's cached reference data.

    ``population`` and ``employment_proxy`` are independently nullable:
    Census and QCEW data coverage is asymmetric (see R2 in
    ``specs/069-sqlite-cache-optimization/research.md``). The four
    combinations ``{(present, present), (present, None), (None, present),
    (None, None)}`` are all legitimate empirical states.

    Frozen: once constructed at hydrate time, never mutated.
    """

    model_config = ConfigDict(frozen=True)

    population: int | None = Field(default=None, ge=0)
    employment_proxy: float | None = Field(default=None, ge=0.0)


class ReferenceDataCache:
    """Per-bridge reference-data cache.

    Contract: :doc:`/specs/069-sqlite-cache-optimization/contracts/reference_data_cache_contract`.

    Lifecycle:

    1. Construction: records the SQLite path. No connection opened.
    2. ``hydrate``: opens one connection, issues two batched SQL
       queries (Census + QCEW), populates the per-tuple entry dict,
       sets ``_hydrated = True``. One-shot — calling twice raises.
    3. ``lookup_*``: returns cached values. No DB I/O.
    4. ``mark_*_miss_logged``: tracks first-occurrence semantics for
       missing-data warnings (FR-004 / SC-004).
    """

    def __init__(self, sqlite_path: Path) -> None:
        self._sqlite_path: Path = sqlite_path
        self._entries: dict[tuple[str, int], ReferenceCacheEntry] = {}
        self._population_db_reads: int = 0
        self._employment_db_reads: int = 0
        self._population_miss_logged: set[tuple[str, int]] = set()
        self._employment_miss_logged: set[tuple[str, int]] = set()
        self._hydrated: bool = False

    @property
    def population_db_reads(self) -> int:
        return self._population_db_reads

    @property
    def employment_db_reads(self) -> int:
        return self._employment_db_reads

    @property
    def total_db_reads(self) -> int:
        return self._population_db_reads + self._employment_db_reads

    def hydrate(
        self,
        scope_fips: frozenset[str],
        year_set: frozenset[int],
    ) -> None:
        """Populate the cache by batched SQL against the SQLite reference DB.

        Per data-model.md §2 algorithm. Issues two queries under a single
        ``sqlite3.Connection`` (Census-by-(fips, year) and
        QCEW-by-(fips, year)); the QCEW result feeds both the
        Census-missing population fallback AND the employment proxy.

        Args:
            scope_fips: 5-digit FIPS codes for the scope counties.
                Must be non-empty.
            year_set:   Calendar years in scope. Empty set is allowed
                (degenerate zero-tick run → no-op hydrate).

        Raises:
            RuntimeError: If called twice on the same instance.
            ValueError: If ``scope_fips`` is empty.
            FileNotFoundError: If the SQLite file does not exist.
            sqlite3.Error: Propagated unchanged on any SQL failure.
        """
        if self._hydrated:
            raise RuntimeError("ReferenceDataCache.hydrate called twice")
        if not scope_fips:
            raise ValueError("scope_fips must be non-empty")

        if not year_set:
            # Degenerate zero-tick case: no entries, but cache is "hydrated".
            self._hydrated = True
            return

        if not self._sqlite_path.exists():
            raise FileNotFoundError(f"SQLite reference DB not found: {self._sqlite_path}")

        fips_tuple = tuple(sorted(scope_fips))
        year_tuple = tuple(sorted(year_set))
        fips_placeholders = ",".join(["?"] * len(fips_tuple))
        year_placeholders = ",".join(["?"] * len(year_tuple))

        census_sql = (
            f"SELECT dc.fips, t.year, COALESCE(SUM(fci.household_count), 0) "  # noqa: S608 — count-only placeholder generation; values bound separately
            f"FROM fact_census_income fci "
            f"JOIN dim_county dc ON dc.county_id = fci.county_id "
            f"JOIN dim_time t ON t.time_id = fci.time_id "
            f"WHERE dc.fips IN ({fips_placeholders}) AND t.year IN ({year_placeholders}) "
            f"GROUP BY dc.fips, t.year"
        )
        qcew_sql = (
            f"SELECT dc.fips, t.year, COALESCE(SUM(fq.employment), 0) "  # noqa: S608 — count-only placeholder generation; values bound separately
            f"FROM fact_qcew_annual fq "
            f"JOIN dim_county dc ON dc.county_id = fq.county_id "
            f"JOIN dim_time t ON t.time_id = fq.time_id "
            f"WHERE dc.fips IN ({fips_placeholders}) AND t.year IN ({year_placeholders}) "
            f"GROUP BY dc.fips, t.year"
        )
        params = (*fips_tuple, *year_tuple)

        with sqlite3.connect(self._sqlite_path) as conn:
            census_by_tuple: dict[tuple[str, int], int] = {
                (fips, year): int(total)
                for fips, year, total in conn.execute(census_sql, params).fetchall()
            }
            qcew_by_tuple: dict[tuple[str, int], int] = {
                (fips, year): int(total)
                for fips, year, total in conn.execute(qcew_sql, params).fetchall()
            }

        for fips in fips_tuple:
            for year in year_tuple:
                pop = self._resolve_population(fips, year, census_by_tuple, qcew_by_tuple)
                emp = self._resolve_employment_proxy(fips, year, qcew_by_tuple)
                self._entries[(fips, year)] = ReferenceCacheEntry(
                    population=pop, employment_proxy=emp
                )
                self._population_db_reads += 1
                self._employment_db_reads += 1

        self._hydrated = True

    @staticmethod
    def _resolve_population(
        fips: str,
        year: int,
        census_by_tuple: dict[tuple[str, int], int],
        qcew_by_tuple: dict[tuple[str, int], int],
    ) -> int | None:
        """Census primary + QCEW × 0.33 fallback — matches legacy fetcher.

        Mirrors ``fetch_population_for_county_at_tick`` byte-for-byte:
        Census present and > 0 ⇒ Census; else QCEW > 0 ⇒ int(QCEW × 0.33);
        else None (both missing, fetcher would have raised
        ``ReferenceDataMissingError``).
        """
        census_total = census_by_tuple.get((fips, year), 0)
        if census_total > 0:
            return census_total
        qcew_total = qcew_by_tuple.get((fips, year), 0)
        if qcew_total > 0:
            return int(qcew_total * _QCEW_TO_POP_FALLBACK_RATIO)
        return None

    @staticmethod
    def _resolve_employment_proxy(
        fips: str,
        year: int,
        qcew_by_tuple: dict[tuple[str, int], int],
    ) -> float | None:
        """QCEW only — matches ``fetch_employment_proxy_for_county_at_tick``.

        QCEW SUM(employment) > 0 ⇒ that float; else None (legacy fetcher
        would have raised ``ReferenceDataMissingError``).
        """
        qcew_total = qcew_by_tuple.get((fips, year), 0)
        if qcew_total > 0:
            return float(qcew_total)
        return None

    def lookup_population(self, county_fips: str, year: int) -> int | None:
        """Cached population for ``(county_fips, year)``.

        Returns ``None`` for tuples whose underlying data was absent at
        hydrate time.

        Raises:
            RuntimeError: If the cache is not yet hydrated.
            KeyError: If ``(county_fips, year)`` was not in the hydrated scope.
        """
        if not self._hydrated:
            raise RuntimeError("ReferenceDataCache: not hydrated")
        return self._entries[(county_fips, year)].population

    def lookup_employment_proxy(self, county_fips: str, year: int) -> float | None:
        """Cached employment proxy for ``(county_fips, year)``.

        Returns ``None`` for tuples whose underlying QCEW data was absent.

        Raises:
            RuntimeError: If the cache is not yet hydrated.
            KeyError: If ``(county_fips, year)`` was not in the hydrated scope.
        """
        if not self._hydrated:
            raise RuntimeError("ReferenceDataCache: not hydrated")
        return self._entries[(county_fips, year)].employment_proxy

    def mark_population_miss_logged(self, county_fips: str, year: int) -> bool:
        """Once-per-tuple miss-log marker for the population field.

        Returns ``True`` on the first call for ``(county_fips, year)``
        and ``False`` thereafter. Drives SC-004 (warning at most once
        per tuple, never once per tuple-tick).

        Raises:
            RuntimeError: If the cache is not yet hydrated.
            KeyError: If ``(county_fips, year)`` was not in the hydrated scope.
        """
        if not self._hydrated:
            raise RuntimeError("ReferenceDataCache: not hydrated")
        if (county_fips, year) not in self._entries:
            raise KeyError((county_fips, year))
        if (county_fips, year) in self._population_miss_logged:
            return False
        self._population_miss_logged.add((county_fips, year))
        return True

    def mark_employment_miss_logged(self, county_fips: str, year: int) -> bool:
        """Once-per-tuple miss-log marker for the employment-proxy field.

        Returns ``True`` on the first call for ``(county_fips, year)``
        and ``False`` thereafter.

        Raises:
            RuntimeError: If the cache is not yet hydrated.
            KeyError: If ``(county_fips, year)`` was not in the hydrated scope.
        """
        if not self._hydrated:
            raise RuntimeError("ReferenceDataCache: not hydrated")
        if (county_fips, year) not in self._entries:
            raise KeyError((county_fips, year))
        if (county_fips, year) in self._employment_miss_logged:
            return False
        self._employment_miss_logged.add((county_fips, year))
        return True
