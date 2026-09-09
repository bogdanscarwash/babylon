"""Data contracts on the in-repo Ricci/Hickel unequal-exchange artifacts.

The two CSVs shipped at ``src/babylon/data/reference/`` (owner ruling
2026-07-16: reference data this small travels with the code) are the source
of truth for the game's imperial-rent empirics:

* ``babylon_hickel_final.csv`` — Hickel/Sullivan/Zoomkawala (2021) annual
  drain from the global South via exchange-rate deviation (ERDI), 1960–2017.
  Feeds ``fact_hickel_erdi_annual`` via ``tools/ingest/hickel_erdi.py``.
* ``babylon_ricci_final.csv`` — Ricci (2021, Table 6.2) region-level
  unequal-exchange value transfers, 1995–2009. TOTAL rows feed
  ``fact_ricci_unequal_exchange`` via ``tools/ingest/ricci_unequal.py``.

Every law asserted here was verified against the raw data before being
pinned (0 violations unless stated). These are internal-consistency and
direction-of-extraction contracts on REFERENCE data — they say nothing
about simulation trajectories.
"""

from __future__ import annotations

import csv
from pathlib import Path

import pytest

_REFERENCE_DIR = Path(__file__).resolve().parents[3] / "src" / "babylon" / "data" / "reference"
HICKEL_CSV = _REFERENCE_DIR / "babylon_hickel_final.csv"
RICCI_CSV = _REFERENCE_DIR / "babylon_ricci_final.csv"


@pytest.fixture(scope="module")
def hickel_rows() -> list[dict[str, str]]:
    with HICKEL_CSV.open() as fh:
        return list(csv.DictReader(fh))


@pytest.fixture(scope="module")
def ricci_rows() -> list[dict[str, str]]:
    with RICCI_CSV.open() as fh:
        return list(csv.DictReader(fh))


class TestHickelDrainSeries:
    """Hickel/Sullivan/Zoomkawala (2021) annual-drain internal consistency."""

    def test_coverage_contiguous_1960_2017(self, hickel_rows: list[dict[str, str]]) -> None:
        years = [int(r["year"]) for r in hickel_rows]
        assert len(hickel_rows) == 58
        assert years == list(range(1960, 2018)), "years must be contiguous 1960–2017"

    def test_cumulative_drain_is_running_sum(self, hickel_rows: list[dict[str, str]]) -> None:
        """cumulative_drain(t) == cumulative_drain(t-1) + annual_drain(t), exactly.

        Verified 0 violations across all 58 rows before pinning — this is the
        series' defining accounting identity and the loader's best drift alarm.
        """
        prev = None
        for row in hickel_rows:
            cum = float(row["cumulative_drain"])
            annual = float(row["annual_drain_usd_billions"])
            if prev is not None:
                expected = prev + annual
                assert abs(cum - expected) <= 1e-6 * max(1.0, abs(expected)), (
                    f"{row['year']}: cumulative {cum} != {prev} + {annual}"
                )
            prev = cum

    def test_erdi_always_favors_the_core(self, hickel_rows: list[dict[str, str]]) -> None:
        """ERDI > 1 in every year — exchange-rate deviation never reverses.

        Observed band [2.4, 8.38]; asserted at the theory-grounded floor
        (ERDI = 1 would mean zero drain via price distortion) plus a wide
        observed ceiling.
        """
        for row in hickel_rows:
            erdi = float(row["erdi"])
            assert 1.0 < erdi <= 9.0, f"{row['year']}: ERDI {erdi} outside (1, 9]"

    def test_annual_drain_strictly_positive(self, hickel_rows: list[dict[str, str]]) -> None:
        """The drain never reverses direction: South → North every year."""
        for row in hickel_rows:
            drain = float(row["annual_drain_usd_billions"])
            assert drain > 0.0, f"{row['year']}: non-positive drain {drain}"

    def test_single_canonical_source(self, hickel_rows: list[dict[str, str]]) -> None:
        assert {r["source"] for r in hickel_rows} == {"Hickel_Sullivan_Zoomkawala_2021"}


class TestRicciTransferStructure:
    """Ricci (2021) unequal-exchange transfer sign/direction laws."""

    def test_row_count_and_years(self, ricci_rows: list[dict[str, str]]) -> None:
        assert len(ricci_rows) == 51
        assert {r["year"] for r in ricci_rows} == {"1995", "2000", "2007", "2009"}

    def test_sign_convention(self, ricci_rows: list[dict[str, str]]) -> None:
        """signed_value == +value for INFLOW rows, −value for OUTFLOW rows."""
        for row in ricci_rows:
            value = float(row["value_usd_billions"])
            signed = float(row["signed_value"])
            expected = value if row["flow_direction"] == "INFLOW" else -value
            assert abs(signed - expected) <= 1e-9, (
                f"{row['year']} {row['region_name']} {row['transfer_type']}: "
                f"signed {signed} != {expected}"
            )

    def test_extraction_direction_law(self, ricci_rows: list[dict[str, str]]) -> None:
        """CORE regions only ever receive (INFLOW); the (semi-)periphery only
        ever bleeds (OUTFLOW). Zero violations in the source — this IS the
        unequal-exchange thesis in the data's own structure."""
        for row in ricci_rows:
            if row["region_type"] == "CORE":
                assert row["flow_direction"] == "INFLOW", (
                    f"{row['year']} {row['region_name']}: CORE region with OUTFLOW"
                )
            else:
                assert row["flow_direction"] == "OUTFLOW", (
                    f"{row['year']} {row['region_name']}: {row['region_type']} region with INFLOW"
                )

    def test_gvc_subset_bounded_by_total_except_2007_nonoecd(
        self, ricci_rows: list[dict[str, str]]
    ) -> None:
        """GVC transfers ≤ TOTAL per (year, region) — with ONE pinned anomaly.

        2007 Non-OECD reports GVC 6700 > TOTAL 4900 in the source data. The
        owner's read (2026-07-16): 2007 is the crisis onset (Bear Stearns'
        collapse) — value-chain flows dislocated from totals. Pinned as a
        known data fact, not asserted away; if a future re-transcription
        "fixes" it silently, this test flags the change for review.
        """
        totals: dict[tuple[str, str], float] = {}
        gvcs: dict[tuple[str, str], float] = {}
        for row in ricci_rows:
            key = (row["year"], row["region_name"])
            bucket = totals if row["transfer_type"] == "TOTAL" else gvcs
            bucket[key] = float(row["value_usd_billions"])
        anomalies = {key for key, gvc in gvcs.items() if key in totals and gvc > totals[key] + 1e-9}
        assert anomalies == {("2007", "Non-OECD")}, (
            f"GVC>TOTAL anomaly set changed: {sorted(anomalies)}"
        )

    def test_region_tier_taxonomy_pinned(self, ricci_rows: list[dict[str, str]]) -> None:
        """Every region carries a fixed world-system tier across all years."""
        tiers: dict[str, set[str]] = {}
        for row in ricci_rows:
            tiers.setdefault(row["region_name"], set()).add(row["region_type"])
        inconsistent = {name: sorted(ts) for name, ts in tiers.items() if len(ts) > 1}
        assert not inconsistent, f"regions with shifting tiers: {inconsistent}"
        assert tiers["China"] == {"SEMI_PERIPHERY"}
        assert tiers["Sub-Saharan Africa"] == {"PERIPHERY"}
        assert tiers["North America"] == {"CORE"}
