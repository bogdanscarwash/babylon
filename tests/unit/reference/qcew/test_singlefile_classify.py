"""Spec-086 T009: singlefile row classification + identity resolution (US1).

RED phase until T014 implements ``babylon_data.qcew.singlefile``.

Covers research D7 (county identity) and FR-013/FR-014: routing by agglvl,
pseudo-area exclusions with per-class counters, Shannon→Oglala mapping
(incl. the 2015 both-published dedupe), hard errors on unknown non-pseudo
fips and header drift.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from tests.fixtures.qcew import (
    SINGLEFILE_HEADER,
    constraint_70_row,
    constraint_71_row,
    leaf_row,
    naics_constraint_row,
    us_total_row,
    write_mini_singlefile,
)

singlefile = pytest.importorskip(
    "babylon_data.qcew.singlefile",
    reason="babylon-data symlink not resolved (CI)",
)

pytestmark = [pytest.mark.unit, pytest.mark.ledger]

KNOWN_FIPS = {"26163", "26099", "46102", "46113", "09110"}


def _read(tmp_path: Path, rows: list[str], year: int = 2010):  # type: ignore[no-untyped-def]
    path = write_mini_singlefile(tmp_path, year, rows)
    return singlefile.read_singlefile(path, year=year, known_county_fips=KNOWN_FIPS)


class TestRouting:
    def test_agglvl_routing(self, tmp_path: Path) -> None:
        data = _read(
            tmp_path,
            [
                constraint_70_row("26163", estabs=10, employment=1000, wages=50_000_000),
                constraint_71_row("26163", "5", estabs=9, employment=900, wages=45_000_000),
                naics_constraint_row(
                    "26163", "5", "31-33", estabs=9, employment=900, wages=45_000_000
                ),
                naics_constraint_row(
                    "26163", "5", "336", estabs=9, employment=900, wages=45_000_000
                ),
                naics_constraint_row(
                    "26163", "5", "3361", estabs=9, employment=900, wages=45_000_000
                ),
                naics_constraint_row(
                    "26163", "5", "33611", estabs=9, employment=900, wages=45_000_000
                ),
                leaf_row("26163", "5", "336111", estabs=5, employment=600, wages=30_000_000),
                leaf_row("26163", "5", "336112", estabs=4, suppressed=True),
                us_total_row(estabs=100, employment=10_000, wages=500_000_000),
            ],
        )
        assert len(data.leaves) == 2
        assert tuple({row.area_fips for row in data.leaves}) == ("26163",)
        assert data.constraints_70["26163"].employment == 1000
        assert data.constraints_71[("26163", "5")].employment == 900
        assert ("26163", "5", "31-33") in data.constraints_naics
        assert ("26163", "5", "33611") in data.constraints_naics
        assert data.national_total is not None
        assert data.national_total.employment == 10_000

    def test_suppressed_leaf_semantics(self, tmp_path: Path) -> None:
        data = _read(tmp_path, [leaf_row("26163", "5", "336112", estabs=4, suppressed=True)])
        (row,) = data.leaves
        assert row.suppressed is True
        assert row.estabs == 4
        assert row.employment == 0
        assert row.wages == 0

    def test_header_mismatch_is_hard_error(self, tmp_path: Path) -> None:
        path = tmp_path / "2010.annual.singlefile.csv"
        bad_header = SINGLEFILE_HEADER.replace("area_fips", "area_flps")
        path.write_text(bad_header + "\n", encoding="utf-8")
        with pytest.raises(singlefile.HeaderMismatchError):
            singlefile.read_singlefile(path, year=2010, known_county_fips=KNOWN_FIPS)


class TestExclusions:
    def test_pseudo_area_classes_counted(self, tmp_path: Path) -> None:
        rows = [
            us_total_row(estabs=1, employment=1, wages=1),
            leaf_row("26163", "5", "336111", estabs=1, employment=1, wages=1),
        ]
        # Non-county rows the classifier must exclude, one per class:
        rows.append(
            leaf_row("US000", "5", "336111", estabs=1, employment=1, wages=1)
        )  # US-level detail
        rows.append(leaf_row("USMSA", "5", "336111", estabs=1, employment=1, wages=1))
        rows.append(leaf_row("C1002", "5", "336111", estabs=1, employment=1, wages=1))  # MSA
        rows.append(leaf_row("CS102", "5", "336111", estabs=1, employment=1, wages=1))  # CSA
        rows.append(leaf_row("26000", "5", "336111", estabs=1, employment=1, wages=1))  # statewide
        rows.append(leaf_row("26999", "5", "336111", estabs=1, employment=1, wages=1))  # SS999
        rows.append(leaf_row("78010", "5", "336111", estabs=1, employment=1, wages=1))  # VI
        data = _read(tmp_path, rows)
        assert len(data.leaves) == 1
        counts = data.exclusions
        assert counts[singlefile.ExclusionClass.US_NATIONAL] == 2
        assert counts[singlefile.ExclusionClass.MSA] == 2  # C#### MSA + CS### CSA
        assert counts[singlefile.ExclusionClass.STATEWIDE] == 1
        assert counts[singlefile.ExclusionClass.SS999_UNKNOWN_COUNTY] == 1
        assert counts[singlefile.ExclusionClass.FIPS_NOT_IN_DIM_COUNTY] == 1

    def test_agglvl_72_73_ignored_but_counted(self, tmp_path: Path) -> None:
        raw = leaf_row("26163", "5", "336111", estabs=1, employment=1, wages=1)
        domain_row = raw.replace('"78"', '"72"', 1)
        supersector_row = raw.replace('"78"', '"73"', 1)
        data = _read(tmp_path, [domain_row, supersector_row])
        assert data.leaves == []
        assert data.exclusions[singlefile.ExclusionClass.AGGLVL_72_73_UNUSED] == 2

    def test_unknown_non_pseudo_fips_halts(self, tmp_path: Path) -> None:
        with pytest.raises(singlefile.UnknownCountyError, match="99123"):
            _read(tmp_path, [leaf_row("99123", "5", "336111", estabs=1, employment=1, wages=1)])


class TestIdentityResolution:
    def test_shannon_maps_to_oglala_when_only_shannon_published(self, tmp_path: Path) -> None:
        data = _read(
            tmp_path,
            [leaf_row("46113", "5", "336111", estabs=2, employment=50, wages=1_000_000)],
            year=2010,
        )
        (row,) = data.leaves
        assert row.area_fips == "46102"
        assert data.identity.shannon_mapped_rows == 1
        assert data.identity.shannon_2015_duplicates_dropped == 0

    def test_2015_both_published_keeps_oglala_drops_shannon(self, tmp_path: Path) -> None:
        data = _read(
            tmp_path,
            [
                leaf_row(
                    "46102", "5", "336111", estabs=2, employment=52, wages=1_040_000, year=2015
                ),
                leaf_row(
                    "46113", "5", "336111", estabs=2, employment=50, wages=1_000_000, year=2015
                ),
                constraint_70_row("46102", estabs=2, employment=52, wages=1_040_000, year=2015),
                constraint_70_row("46113", estabs=2, employment=50, wages=1_000_000, year=2015),
            ],
            year=2015,
        )
        assert {row.area_fips for row in data.leaves} == {"46102"}
        assert {row.employment for row in data.leaves} == {52}
        (leaf,) = data.leaves
        assert leaf.employment == 52  # the kept 46102 magnitudes, not Shannon's
        assert set(data.constraints_70) == {"46102"}
        assert data.identity.shannon_2015_duplicates_dropped == 2
        assert data.exclusions[singlefile.ExclusionClass.DUPLICATE_IDENTITY] == 2

    def test_ct_2024_planning_region_loads_as_published(self, tmp_path: Path) -> None:
        data = _read(
            tmp_path,
            [leaf_row("09110", "5", "336111", estabs=3, employment=70, wages=2_000_000, year=2024)],
            year=2024,
        )
        (row,) = data.leaves
        assert row.area_fips == "09110"
