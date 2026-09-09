"""Owner-approved empirical-law contracts: β, US labor share, imperial-rent multiple.

Owner ruling 2026-07-16 approved encoding three empirical laws as tests. The
per-year series ship in-repo at ``src/babylon/data/reference/`` (derived from
the babylon-data drive by ``tools/extract/empirical_invariant_series.py`` —
CI never touches the drive; the extraction was cross-checked against two
independent computations before landing):

* ``wid_us_beta.csv`` — Piketty's β = US national wealth / national income
  (WID ``wnweali999``), 225 continuous years 1800–2024.
* ``bea_us_labor_share.csv`` — aggregate Compensation/Value-Added from the
  BEA Use_Summary workbook (V001/VABAS over industry columns), 1997–2024.
  This is W_c/V_c economy-wide: the Fundamental Theorem's empirical anchor.
* ``wid_imperial_rent_multiple.csv`` — US per-adult income vs Sub-Saharan
  Africa's mean (WID XF-MER aggregate — never single-country LCU/FX, a
  documented 17× trap), plus the labor-aristocracy anchor: the US BOTTOM-50%
  average against the SSA mean, 1950–2024.

**Conditionality (owner ruling):** laws of the capitalist mode of production,
pinning REFERENCE data. Never assert them against simulation trajectories —
in-game rupture legitimately breaks them.
"""

from __future__ import annotations

import csv
from pathlib import Path

import pytest

_REFERENCE_DIR = Path(__file__).resolve().parents[3] / "src" / "babylon" / "data" / "reference"


def _read(name: str) -> list[dict[str, str]]:
    with (_REFERENCE_DIR / name).open() as fh:
        return list(csv.DictReader(fh))


@pytest.fixture(scope="module")
def beta_rows() -> list[dict[str, str]]:
    return _read("wid_us_beta.csv")


@pytest.fixture(scope="module")
def labor_share_rows() -> list[dict[str, str]]:
    return _read("bea_us_labor_share.csv")


@pytest.fixture(scope="module")
def multiple_rows() -> list[dict[str, str]]:
    return _read("wid_imperial_rent_multiple.csv")


class TestWealthIncomeRatioBeta:
    """Piketty's β stays in a bounded band across 225 continuous years."""

    def test_coverage_contiguous_1800_2024(self, beta_rows: list[dict[str, str]]) -> None:
        years = [int(r["year"]) for r in beta_rows]
        assert years == list(range(1800, 2025))

    def test_full_record_band(self, beta_rows: list[dict[str, str]]) -> None:
        """β ∈ [2, 7] every single year on record (observed [2.49, 6.22])."""
        for row in beta_rows:
            beta = float(row["beta"])
            assert 2.0 <= beta <= 7.0, f"{row['year']}: β={beta} outside [2, 7]"

    def test_modern_era_band_and_rise(self, beta_rows: list[dict[str, str]]) -> None:
        """2000+: β ∈ [3.5, 6.5]; ≥ 4 every year from 2012 (Piketty's thesis)."""
        for row in beta_rows:
            year, beta = int(row["year"]), float(row["beta"])
            if year >= 2000:
                assert 3.5 <= beta <= 6.5, f"{year}: β={beta} outside modern band"
            if year >= 2012:
                assert beta >= 4.0, f"{year}: β={beta} < 4 contradicts the modern-era rise"


class TestAggregateUSLaborShare:
    """W_c < V_c economy-wide, every year on record — the Fundamental Theorem's anchor."""

    def test_coverage_contiguous_1997_2024(self, labor_share_rows: list[dict[str, str]]) -> None:
        years = [int(r["year"]) for r in labor_share_rows]
        assert years == list(range(1997, 2025))

    def test_labor_share_column_is_derived(self, labor_share_rows: list[dict[str, str]]) -> None:
        """labor_share == compensation / value_added, re-derived per row."""
        for row in labor_share_rows:
            comp = float(row["compensation_millions_usd"])
            va = float(row["value_added_millions_usd"])
            assert float(row["labor_share"]) == pytest.approx(comp / va, abs=1e-6)

    def test_band_every_year(self, labor_share_rows: list[dict[str, str]]) -> None:
        """Labor share ∈ [0.52, 0.61] each year (observed [0.5302, 0.5923])."""
        for row in labor_share_rows:
            share = float(row["labor_share"])
            assert 0.52 <= share <= 0.61, f"{row['year']}: labor share {share} outside band"

    def test_wages_never_reach_value_produced(self, labor_share_rows: list[dict[str, str]]) -> None:
        """W_c < V_c strictly, every year — aggregate compensation has never
        approached total value added; the gap IS the surplus."""
        for row in labor_share_rows:
            assert float(row["labor_share"]) < 1.0

    def test_secular_decline(self, labor_share_rows: list[dict[str, str]]) -> None:
        """The declining-labor-share era: 2015–2024 mean < 1997–2006 mean."""
        by_year = {int(r["year"]): float(r["labor_share"]) for r in labor_share_rows}
        early = sum(by_year[y] for y in range(1997, 2007)) / 10
        late = sum(by_year[y] for y in range(2015, 2025)) / 10
        assert late < early, f"labor share did not decline: {early:.4f} → {late:.4f}"


class TestImperialRentMultiple:
    """The core:periphery income gap is persistent — no convergence in 75 years."""

    def test_coverage_contiguous_1950_2024(self, multiple_rows: list[dict[str, str]]) -> None:
        years = [int(r["year"]) for r in multiple_rows]
        assert years == list(range(1950, 2025))

    def test_multiples_are_derived(self, multiple_rows: list[dict[str, str]]) -> None:
        for row in multiple_rows:
            us = float(row["us_mean_income_per_adult_usd"])
            b50 = float(row["us_bottom50_avg_income_usd"])
            ssa = float(row["ssa_mean_income_per_adult_usd"])
            assert float(row["multiple_us_mean_over_ssa_mean"]) == pytest.approx(us / ssa, rel=1e-3)
            assert float(row["multiple_us_bottom50_over_ssa_mean"]) == pytest.approx(
                b50 / ssa, rel=1e-3
            )

    def test_imperial_multiple_band(self, multiple_rows: list[dict[str, str]]) -> None:
        """US mean : SSA mean ∈ [10, 40] every year (observed [18.0, 33.1])."""
        for row in multiple_rows:
            multiple = float(row["multiple_us_mean_over_ssa_mean"])
            assert 10.0 <= multiple <= 40.0, (
                f"{row['year']}: imperial multiple {multiple} outside [10, 40]"
            )

    def test_labor_aristocracy_anchor_band(self, multiple_rows: list[dict[str, str]]) -> None:
        """Even the poorest half of the core out-earns the periphery MEAN by
        3–5.5× every year for 75 years (observed [3.36, 5.02], stdev 0.45 —
        the tightest band in the whole investigation). The material basis of
        the labor aristocracy."""
        for row in multiple_rows:
            anchor = float(row["multiple_us_bottom50_over_ssa_mean"])
            assert 3.0 <= anchor <= 5.5, (
                f"{row['year']}: labor-aristocracy anchor {anchor} outside [3, 5.5]"
            )

    def test_no_secular_convergence(self, multiple_rows: list[dict[str, str]]) -> None:
        """The gap does not close: the last 25 years' mean multiple is no
        smaller than the first 25 years' (1950–1974). Emmanuel/Amin hold."""
        by_year = {
            int(r["year"]): float(r["multiple_us_mean_over_ssa_mean"]) for r in multiple_rows
        }
        early = sum(by_year[y] for y in range(1950, 1975)) / 25
        late = sum(by_year[y] for y in range(2000, 2025)) / 25
        assert late >= early, f"core-periphery gap converged: {early:.2f} → {late:.2f}"
