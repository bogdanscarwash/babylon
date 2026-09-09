"""Redundant-source wealth-concentration contract: Fed DFA vs the WID bands.

``fact_fred_wealth_shares`` ships the Federal Reserve Distributional Financial
Accounts net-worth shares (SCF-benchmarked — methodologically independent of
the WID/Piketty tax-data series) for the four population-quantile brackets,
2010Q1–2024Q4. Until 2026-07-16 the table was fully populated but orphaned
(``tools/make_reference_subset.py`` tags it unreferenced); these tests make it
a consumed, load-bearing corroboration source for the wealth-distribution
invariants pinned in ``tests/unit/config/test_wealth_distribution_invariants.py``.

The bands asserted here are the observed 2010Q1–2024Q4 extremes ±1pp headroom
for future FRED revisions; a re-ingest that moves outside them means either a
major upstream revision (verify, then widen deliberately) or a broken loader.

**Conditionality (owner ruling, 2026-07-16):** these are laws of the
capitalist mode of production pinning REFERENCE DATA (the real, pre-rupture
United States). They must never be asserted against live simulation output.
"""

from __future__ import annotations

import sqlite3
from pathlib import Path

import pytest

SQLITE_REF = Path("data/sqlite/marxist-data-3NF.sqlite")

#: FRED DFA "Share of Net Worth Held by ..." series, as ingested.
NET_WORTH_SERIES = ("WFRBST01134", "WFRBSN09161", "WFRBSN40188", "WFRBSB50215")

#: percentile_code → (observed 2010Q1–2024Q4 min/max ±1pp, babylon_class).
#: Percent units, matching ``share_percent``.
DFA_BANDS: dict[str, tuple[float, float, str]] = {
    "LT01": (27.1, 32.0, "core_bourgeoisie"),  # top 1%: [28.1, 31.0]
    "N09": (35.4, 41.1, "petty_bourgeoisie"),  # p90–99: [36.4, 40.1]
    "N40": (27.1, 32.6, "labor_aristocracy"),  # p50–90: [28.1, 31.6]
    "B50": (0.0, 3.7, "internal_proletariat"),  # bottom 50%: [0.4, 2.7]
}

EXPECTED_QUARTERS = 60  # 2010Q1 .. 2024Q4

pytestmark = [
    pytest.mark.requires_reference_db,
    pytest.mark.skipif(not SQLITE_REF.exists(), reason=f"reference DB missing at {SQLITE_REF}"),
]


@pytest.fixture(scope="module")
def dfa_rows() -> list[tuple[str, str, int, int, float]]:
    """(percentile_code, babylon_class, year, quarter, share_percent) rows."""
    with sqlite3.connect(f"file:{SQLITE_REF}?mode=ro", uri=True) as conn:
        rows = conn.execute(
            """
            SELECT w.percentile_code, w.babylon_class, t.year, t.quarter,
                   f.share_percent
            FROM fact_fred_wealth_shares f
            JOIN dim_wealth_class w ON w.wealth_class_id = f.wealth_class_id
            JOIN dim_fred_series s ON s.series_id = f.series_id
            JOIN dim_time t ON t.time_id = f.time_id
            WHERE s.series_code IN (?, ?, ?, ?)
            """,
            NET_WORTH_SERIES,
        ).fetchall()
    return [(str(r[0]), str(r[1]), int(r[2]), int(r[3]), float(r[4])) for r in rows]


def test_full_quarterly_coverage(dfa_rows: list[tuple[str, str, int, int, float]]) -> None:
    """Every bracket carries all 60 quarters, 2010Q1 through 2024Q4."""
    by_code: dict[str, set[tuple[int, int]]] = {}
    for code, _cls, year, quarter, _pct in dfa_rows:
        by_code.setdefault(code, set()).add((year, quarter))
    assert set(by_code) == set(DFA_BANDS), f"bracket codes drifted: {sorted(by_code)}"
    for code, quarters in by_code.items():
        assert len(quarters) == EXPECTED_QUARTERS, (
            f"{code}: {len(quarters)} quarters, expected {EXPECTED_QUARTERS}"
        )
        assert min(quarters) == (2010, 1) and max(quarters) == (2024, 4)


def test_shares_within_dfa_bands(dfa_rows: list[tuple[str, str, int, int, float]]) -> None:
    """Every quarterly share sits inside its bracket's observed band."""
    for code, _cls, year, quarter, pct in dfa_rows:
        low, high, _ = DFA_BANDS[code]
        assert low <= pct <= high, (
            f"{code} {year}Q{quarter}: share {pct}% outside band [{low}, {high}]"
        )


def test_bottom50_strictly_positive(
    dfa_rows: list[tuple[str, str, int, int, float]],
) -> None:
    """DFA refinement of the WID law: bottom-50% share is small but > 0.

    WID's tax-data series dips negative post-2008 (underwater mortgages);
    the SCF-benchmarked DFA never does. Both agree it is never material.
    """
    bottom = [pct for code, _cls, _y, _q, pct in dfa_rows if code == "B50"]
    assert bottom, "no B50 rows"
    assert all(pct > 0.0 for pct in bottom), f"bottom-50 share non-positive: min={min(bottom)}"


def test_quarterly_shares_partition_total_wealth(
    dfa_rows: list[tuple[str, str, int, int, float]],
) -> None:
    """Per quarter, the four brackets sum to 100% (observed residual ≤ 0.1pp)."""
    sums: dict[tuple[int, int], float] = {}
    for _code, _cls, year, quarter, pct in dfa_rows:
        sums[(year, quarter)] = sums.get((year, quarter), 0.0) + pct
    assert len(sums) == EXPECTED_QUARTERS
    for (year, quarter), total in sums.items():
        assert abs(total - 100.0) <= 0.3, f"{year}Q{quarter}: shares sum to {total}"


def test_babylon_class_mapping_pinned(
    dfa_rows: list[tuple[str, str, int, int, float]],
) -> None:
    """dim_wealth_class's bracket → babylon class mapping is load-bearing."""
    seen = {(code, cls) for code, cls, _y, _q, _p in dfa_rows}
    expected = {(code, cls) for code, (_lo, _hi, cls) in DFA_BANDS.items()}
    assert seen == expected, f"bracket→class mapping drifted: {sorted(seen)}"
