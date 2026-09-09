"""Unit tests for the national-incidence artifact generator's T4 emission
pipeline (#334 Phase 0, T4) — ``tools/make_national_incidence_artifact.py``.

T4's scope (plan §3 steps 3-10): wires every landed guard (G1/G2/G3/G4/G5/
G6/G8 from T3a/T3b, G7 from T1) into the full per-county measure pipeline
and emits A2 (``national_incidence_county_pole.csv.gz``) and A3
(``national_reproduction_floor.csv``) deterministically. Fixture- and
text-only — the real-DB run is T6's job (environment note, plan §5).

The fixture universe (``FIXTURE_UNIVERSE``, 6 engine fips) is hand-built to
exercise every absence class through the FULL pipeline in one pass:

- ``01001`` — all 4 poles PRESENT (the clean baseline row).
- ``01003`` — C (Indigenous/AIAN, race_id 4) pole ZERO_DENOMINATOR (u=0).
- ``01005`` — I (Chicano/Hispanic, race_id 10) pole SUPPRESSED (u=100, b=0).
- ``01007`` — no cells at all pulled -> all 4 poles ROW_ABSENT.
- ``51515`` — Bedford city VA, the SOLE recoverable A1 crosswalk row
  (-> query fips ``51019``, real cell data lives there): exercises the
  A1/G7 wiring obligation.
- ``46102`` — Pine Ridge, the real A1 DECLARED_HOLE row: all 4 poles
  DECLARED_HOLE, no cells.

Every populated county's category_id=1 (universe) row set exactly satisfies
G6 (``T == Σ(A..G)``, ``H <= A``, ``I <= T``) — verified by hand before
writing the fixture (see the module docstring's arithmetic below each
class). Pooled p̄/q̄ and every downstream figure were independently computed
by hand (not by re-running the module) before being pinned into these
tests — see the T4 task report for the full derivation.
"""

from __future__ import annotations

import csv
import gzip
import sys
from pathlib import Path

import pytest

pytestmark = [pytest.mark.unit]

_REPO_ROOT = Path(__file__).resolve().parents[3]
_TOOLS_DIR = _REPO_ROOT / "tools"
_FIXTURES_DIR = _REPO_ROOT / "tests/fixtures/national_incidence"
sys.path.insert(0, str(_TOOLS_DIR))

import make_national_incidence_artifact as nia  # type: ignore[import-not-found]  # noqa: E402

FIXTURE_UNIVERSE: frozenset[str] = frozenset({"01001", "01003", "01005", "01007", "51515", "46102"})

# Hand-computed pooled reference rates for FIXTURE_UNIVERSE (see module
# docstring) — settler (H) pool: Σb=255, Σu=2200; oppressed (B+C+I) pool:
# Σb=218, Σu=1220.
_EXPECTED_P_BAR = 255 / 2200
_EXPECTED_Q_BAR = 218 / 1220


def _load_fixture_cells() -> tuple[nia.PovertyCell, ...]:
    path = _FIXTURES_DIR / "t4_emission_cells.csv"
    cells: list[nia.PovertyCell] = []
    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        for row in reader:
            cells.append(
                nia.PovertyCell(
                    fips=row["fips"],
                    category_id=int(row["category_id"]),
                    race_id=int(row["race_id"]),
                    person_count=int(row["person_count"]),
                )
            )
    return tuple(cells)


def _fixture_pipeline_inputs() -> tuple[
    dict[str, dict[int, nia.PoleCellPair]], tuple[nia.CrosswalkRow, ...]
]:
    cells = _load_fixture_cells()
    by_race = nia.build_county_race_totals(cells)
    crosswalk_rows = nia.load_crosswalk()
    return by_race, crosswalk_rows


# ---------------------------------------------------------------------------
# Pole map — the pole-letter trap regression guard.
# ---------------------------------------------------------------------------


class TestPoleMap:
    def test_pole_letter_trap_never_swapped(self) -> None:
        """Chicano -> census I (race_id 10); Indigenous -> census C
        (race_id 4). Swapping these is exactly the trap the module
        docstring names."""
        assert nia.POLE_RACE_ID["I"] == 10
        assert nia.POLE_RACE_ID["C"] == 4
        assert nia.POLE_RACE_ID["B"] == 3
        assert nia.POLE_RACE_ID["H"] == 9

    def test_pole_roles(self) -> None:
        assert nia.POLE_ROLE["B"] == "oppressed"
        assert nia.POLE_ROLE["C"] == "oppressed"
        assert nia.POLE_ROLE["I"] == "oppressed"
        assert nia.POLE_ROLE["H"] == "settler_reference"


# ---------------------------------------------------------------------------
# build_county_race_totals — the category_id=1/category_id=2 join.
# ---------------------------------------------------------------------------


class TestBuildCountyRaceTotals:
    def test_joins_matching_category_rows(self) -> None:
        cells = _load_fixture_cells()
        by_race = nia.build_county_race_totals(cells)

        assert by_race["01001"][1] == nia.PoleCellPair(universe_u=1000, below_b=150)
        assert by_race["01001"][3] == nia.PoleCellPair(universe_u=200, below_b=40)
        assert set(by_race) == {"01001", "01003", "01005", "51019"}

    def test_raises_on_unpaired_category_row(self) -> None:
        cells = (nia.PovertyCell(fips="99999", category_id=1, race_id=1, person_count=10),)

        with pytest.raises(nia.ArtifactGenerationError, match="missing one"):
            nia.build_county_race_totals(cells)

    def test_raises_on_unexpected_category_id(self) -> None:
        cells = (nia.PovertyCell(fips="99999", category_id=3, race_id=1, person_count=10),)

        with pytest.raises(nia.ArtifactGenerationError, match="category_id"):
            nia.build_county_race_totals(cells)

    def test_raises_on_below_b_exceeding_universe_u(self) -> None:
        """Malformed-cell integrity check (T5 pre-step): a joined pair
        where below-poverty exceeds the universe it is drawn from must
        abort loudly — not a numbered G1-G8 guard, just plain data
        integrity on the raw join."""
        cells = (
            nia.PovertyCell(fips="99999", category_id=1, race_id=1, person_count=10),
            nia.PovertyCell(fips="99999", category_id=2, race_id=1, person_count=15),
        )

        with pytest.raises(nia.ArtifactGenerationError, match="exceeds universe_u"):
            nia.build_county_race_totals(cells)


# ---------------------------------------------------------------------------
# run_t_pole_exactness_for_county — G6 wired on the real joined shape.
# ---------------------------------------------------------------------------


class TestRunTPoleExactnessForCounty:
    def test_passes_on_a_consistent_county(self) -> None:
        by_race, _ = _fixture_pipeline_inputs()

        nia.run_t_pole_exactness_for_county("01001", by_race["01001"])  # no raise == pass

    def test_raises_on_an_inconsistent_county(self) -> None:
        by_race, _ = _fixture_pipeline_inputs()
        broken = dict(by_race["01001"])
        broken[2] = nia.PoleCellPair(universe_u=601, below_b=80)  # A off by 1 -> T != Σ(A..G)

        with pytest.raises(nia.ArtifactGenerationError, match="residual"):
            nia.run_t_pole_exactness_for_county("01001", broken)

    def test_raises_when_a_required_race_id_is_missing(self) -> None:
        by_race, _ = _fixture_pipeline_inputs()
        partial = {k: v for k, v in by_race["01001"].items() if k != 10}  # drop I

        with pytest.raises(nia.ArtifactGenerationError, match="missing"):
            nia.run_t_pole_exactness_for_county("01001", partial)


# ---------------------------------------------------------------------------
# classify_universe_poles — steps 1(A1)+3(G6)+4(G4) assembled.
# ---------------------------------------------------------------------------


class TestClassifyUniversePoles:
    def _classified_by_key(self) -> dict[tuple[str, str], nia.ClassifiedCell]:
        by_race, crosswalk_rows = _fixture_pipeline_inputs()
        classified = nia.classify_universe_poles(FIXTURE_UNIVERSE, by_race, crosswalk_rows)
        return {(c.engine_fips, c.pole): c for c in classified}

    def test_row_count_is_universe_times_four_poles(self) -> None:
        by_race, crosswalk_rows = _fixture_pipeline_inputs()

        classified = nia.classify_universe_poles(FIXTURE_UNIVERSE, by_race, crosswalk_rows)

        assert len(classified) == len(FIXTURE_UNIVERSE) * 4

    def test_01001_all_four_poles_present(self) -> None:
        by_key = self._classified_by_key()

        for pole in nia.POLE_LETTERS:
            assert by_key[("01001", pole)].absence_class == "PRESENT"

    def test_01003_c_pole_is_zero_denominator_others_present(self) -> None:
        by_key = self._classified_by_key()

        assert by_key[("01003", "C")].absence_class == "ZERO_DENOMINATOR"
        assert by_key[("01003", "C")].rate is None
        assert by_key[("01003", "B")].absence_class == "PRESENT"
        assert by_key[("01003", "I")].absence_class == "PRESENT"
        assert by_key[("01003", "H")].absence_class == "PRESENT"

    def test_01005_i_pole_is_suppressed_others_present(self) -> None:
        by_key = self._classified_by_key()

        assert by_key[("01005", "I")].absence_class == "SUPPRESSED"
        assert by_key[("01005", "I")].rate is None
        assert by_key[("01005", "B")].absence_class == "PRESENT"
        assert by_key[("01005", "C")].absence_class == "PRESENT"
        assert by_key[("01005", "H")].absence_class == "PRESENT"

    def test_01007_whole_county_is_row_absent_every_pole(self) -> None:
        by_key = self._classified_by_key()

        for pole in nia.POLE_LETTERS:
            classified = by_key[("01007", pole)]
            assert classified.absence_class == "ROW_ABSENT"
            assert classified.rate is None

    def test_46102_pine_ridge_is_declared_hole_every_pole(self) -> None:
        by_key = self._classified_by_key()

        for pole in nia.POLE_LETTERS:
            assert by_key[("46102", pole)].absence_class == "DECLARED_HOLE"

    def test_51515_bedford_resolves_through_the_crosswalk(self) -> None:
        by_key = self._classified_by_key()
        crosswalk_rows = nia.load_crosswalk()
        bedford_row = next(r for r in crosswalk_rows if r.fips_engine == "51515")

        for pole in nia.POLE_LETTERS:
            classified = by_key[("51515", pole)]
            assert classified.absence_class == "PRESENT"
            assert classified.query_fips == "51019"
            assert classified.fips_source_vintage == bedford_row.vintage_note
            assert classified.fips_source_vintage != nia.NATIVE_VINTAGE_LABEL


class TestG7NonNativeVintageResolvesToACrosswalkRow:
    """Obligation #2: every non-native fips_source_vintage in the emitted
    A2 resolves through the crosswalk (G7's law) — only recoverable=True
    rows substitute."""

    def test_every_non_native_vintage_traces_to_a_recoverable_a1_row(self) -> None:
        by_race, crosswalk_rows = _fixture_pipeline_inputs()
        classified = nia.classify_universe_poles(FIXTURE_UNIVERSE, by_race, crosswalk_rows)
        by_engine = {row.fips_engine: row for row in crosswalk_rows}
        non_native = [c for c in classified if c.fips_source_vintage != nia.NATIVE_VINTAGE_LABEL]

        assert non_native, "fixture must exercise at least one non-native row (Bedford)"
        for c in non_native:
            row = by_engine.get(c.engine_fips)
            assert row is not None, f"{c.engine_fips}: no A1 row for a non-native vintage"
            assert row.recoverable is True
            assert row.vintage_note == c.fips_source_vintage

    def test_only_bedford_is_non_native_in_this_fixture(self) -> None:
        by_race, crosswalk_rows = _fixture_pipeline_inputs()
        classified = nia.classify_universe_poles(FIXTURE_UNIVERSE, by_race, crosswalk_rows)
        non_native_fips = {
            c.engine_fips for c in classified if c.fips_source_vintage != nia.NATIVE_VINTAGE_LABEL
        }

        assert non_native_fips == {"51515"}


# ---------------------------------------------------------------------------
# compute_pooled_ratios (G1) / compute_w — the pooled reference rates and
# the signed witness.
# ---------------------------------------------------------------------------


class TestComputePooledRatios:
    def test_matches_hand_computed_pooled_ratios(self) -> None:
        by_race, crosswalk_rows = _fixture_pipeline_inputs()
        classified = nia.classify_universe_poles(FIXTURE_UNIVERSE, by_race, crosswalk_rows)

        pooled = nia.compute_pooled_ratios(classified)

        assert pooled.p_bar == pytest.approx(_EXPECTED_P_BAR)
        assert pooled.q_bar == pytest.approx(_EXPECTED_Q_BAR)


class TestComputeW:
    def test_matches_the_proposal_formula(self) -> None:
        w = nia.compute_w(below_b=40, universe_u=200, p_bar=_EXPECTED_P_BAR)

        assert w == pytest.approx((40 - 200 * _EXPECTED_P_BAR) / (40 + 200 * _EXPECTED_P_BAR))

    def test_raises_on_zero_denominator(self) -> None:
        with pytest.raises(nia.ArtifactGenerationError, match="denominator"):
            nia.compute_w(below_b=0, universe_u=10, p_bar=0.0)


# ---------------------------------------------------------------------------
# build_county_pole_rows — A2's full assembly.
# ---------------------------------------------------------------------------


class TestBuildCountyPoleRows:
    def _rows_and_by_key(
        self,
    ) -> tuple[tuple[nia.CountyPoleRow, ...], dict[tuple[str, str], nia.CountyPoleRow]]:
        by_race, crosswalk_rows = _fixture_pipeline_inputs()
        rows = nia.build_county_pole_rows(FIXTURE_UNIVERSE, by_race, crosswalk_rows)
        return rows, {(r.fips, r.pole): r for r in rows}

    def test_row_count(self) -> None:
        rows, _ = self._rows_and_by_key()

        assert len(rows) == len(FIXTURE_UNIVERSE) * 4

    def test_rows_sorted_by_fips_then_pole(self) -> None:
        rows, _ = self._rows_and_by_key()

        keys = [(r.fips, r.pole) for r in rows]
        assert keys == sorted(keys)

    def test_01001_b_pole_full_measure_set(self) -> None:
        _, by_key = self._rows_and_by_key()
        row = by_key[("01001", "B")]

        assert row.universe_u == 200
        assert row.below_b == 40
        assert row.rate == pytest.approx(0.2)
        expected_w = (40 - 200 * _EXPECTED_P_BAR) / (40 + 200 * _EXPECTED_P_BAR)
        assert row.w == pytest.approx(expected_w)
        expected_damp = 1 - 1 / (200**0.5)
        assert row.damping_weight == pytest.approx(expected_damp)
        assert row.sigma_damped == pytest.approx(abs(expected_w) * expected_damp)
        expected_mvsn = 40 - 200 * _EXPECTED_P_BAR
        expected_mvdf = 200 * _EXPECTED_Q_BAR - 40
        assert row.mass_vs_settler_norm == pytest.approx(expected_mvsn)
        assert row.mass_vs_demonstrated_floor == pytest.approx(expected_mvdf)
        assert row.lambda_per_capita == pytest.approx(expected_mvsn / 1000)
        assert row.omega_hat_per_capita == pytest.approx(expected_mvdf / 1000)
        assert row.absence_class == "PRESENT"
        assert row.pole_role == "oppressed"
        assert row.fips_source_vintage == "native"

    def test_non_present_rows_have_every_measure_cell_empty(self) -> None:
        _, by_key = self._rows_and_by_key()

        for fips, pole in (("01007", "B"), ("46102", "H"), ("01003", "C"), ("01005", "I")):
            row = by_key[(fips, pole)]
            assert row.universe_u is None
            assert row.below_b is None
            assert row.rate is None
            assert row.w is None
            assert row.sigma_damped is None
            assert row.damping_weight is None
            assert row.mass_vs_settler_norm is None
            assert row.mass_vs_demonstrated_floor is None
            assert row.lambda_per_capita is None
            assert row.omega_hat_per_capita is None

    def test_bedford_row_carries_the_engine_fips_not_the_query_fips(self) -> None:
        _, by_key = self._rows_and_by_key()
        row = by_key[("51515", "B")]

        assert row.fips == "51515"
        assert row.absence_class == "PRESENT"
        assert row.fips_source_vintage != "native"


# ---------------------------------------------------------------------------
# compute_pooled_overlap — G8's pooled disclosure (F2).
# ---------------------------------------------------------------------------


class TestComputePooledOverlap:
    def test_matches_hand_computed_bound(self) -> None:
        by_race, crosswalk_rows = _fixture_pipeline_inputs()
        crosswalk_by_engine = {r.fips_engine: r for r in crosswalk_rows}

        overlap = nia.compute_pooled_overlap(FIXTURE_UNIVERSE, by_race, crosswalk_by_engine)

        # 01001+01003+01005+51515(->51019): ΣA=600+500+550+700=2350;
        # ΣH=550+480+520+650=2200; ΣI=150+100+100+120=470;
        # white_hispanic=2350-2200=150; bound=470-150=320.
        assert overlap.sum_total_a == 2350
        assert overlap.sum_white_non_hispanic_h == 2200
        assert overlap.sum_hispanic_i == 470
        assert overlap.overlap_bound == 320


# ---------------------------------------------------------------------------
# build_reproduction_floor_rows — A3's full assembly (one universe_variant).
# ---------------------------------------------------------------------------


class TestBuildReproductionFloorRows:
    def _rows_and_by_pole(
        self,
    ) -> tuple[tuple[nia.FloorAggregateRow, ...], dict[str, nia.FloorAggregateRow]]:
        by_race, crosswalk_rows = _fixture_pipeline_inputs()
        rows = nia.build_reproduction_floor_rows(
            "artifact", FIXTURE_UNIVERSE, by_race, crosswalk_rows
        )
        return rows, {r.pole: r for r in rows}

    def test_five_rows_four_poles_plus_pooled(self) -> None:
        rows, by_pole = self._rows_and_by_pole()

        assert len(rows) == 5
        assert set(by_pole) == {"B", "C", "I", "H", nia.POOLED_POLE_LABEL}

    def test_universe_variant_label_carries_the_measured_count(self) -> None:
        rows, _ = self._rows_and_by_pole()

        expected_label = f"artifact_{len(FIXTURE_UNIVERSE)}"
        assert all(r.universe_variant == expected_label for r in rows)

    def test_individual_pole_rows_carry_their_own_ratio_of_sums_rate(self) -> None:
        _, by_pole = self._rows_and_by_pole()

        assert by_pole["B"].rate == pytest.approx(130 / 730)
        assert by_pole["C"].rate == pytest.approx(23 / 120)
        assert by_pole["I"].rate == pytest.approx(65 / 370)
        assert by_pole["H"].rate == pytest.approx(255 / 2200)
        assert by_pole["H"].rate == pytest.approx(_EXPECTED_P_BAR)  # H's own rate IS p̄

    def test_individual_pole_rows_leave_cross_pole_columns_empty(self) -> None:
        _, by_pole = self._rows_and_by_pole()

        for pole in ("B", "C", "I", "H"):
            row = by_pole[pole]
            assert row.ratio_bribe_to_deprivation is None
            assert row.overlap_upper_bound is None
            assert row.overlap_bound_share is None

    def test_pooled_row_carries_f1_and_f2_figures(self) -> None:
        _, by_pole = self._rows_and_by_pole()
        pooled = by_pole[nia.POOLED_POLE_LABEL]

        assert pooled.sum_u == 1220
        assert pooled.sum_b == 218
        assert pooled.rate == pytest.approx(_EXPECTED_Q_BAR)
        assert pooled.p_bar == pytest.approx(_EXPECTED_P_BAR)
        assert pooled.q_bar == pytest.approx(_EXPECTED_Q_BAR)
        assert pooled.sum_mass_vs_settler_norm == pytest.approx(76.5909090909091, rel=1e-9)
        assert pooled.sum_mass_vs_demonstrated_floor == pytest.approx(138.11475409836066, rel=1e-9)
        assert pooled.ratio_bribe_to_deprivation == pytest.approx(1.8032786885245902, rel=1e-9)
        assert pooled.overlap_upper_bound == 320
        assert pooled.overlap_bound_share == pytest.approx(320 / 1220, rel=1e-9)

    def test_reconciliation_counts_reconcile_exactly(self) -> None:
        _, by_pole = self._rows_and_by_pole()
        universe_size = len(FIXTURE_UNIVERSE)

        for pole in ("B", "C", "I", "H"):
            row = by_pole[pole]
            assert row.counties_present + row.counties_absent == universe_size
        pooled = by_pole[nia.POOLED_POLE_LABEL]
        assert pooled.counties_present + pooled.counties_absent == 3 * universe_size
        assert pooled.counties_present == 10
        assert pooled.counties_absent == 8


# ---------------------------------------------------------------------------
# F1 sanity — the DoD's "lands near 1.87" claim, on F1's REAL published
# national pooled sums (structural check; the real-DB run is T6's).
# ---------------------------------------------------------------------------


class TestF1SanityWithRealPooledFigures:
    """Feeds F1's real published national pooled sums (plan §1) through the
    same ΣE/ΣΩ mass-summation arithmetic :func:`_pooled_aggregate_row` uses
    — proves the formula reproduces F1's ~1.87 ratio, without a real-DB run
    (fixture-driven per T4's environment note; T6 proves it end-to-end)."""

    def test_ratio_lands_near_1_87_never_1_55(self) -> None:
        sum_u_o, sum_b_o = 103_140_228, 22_634_466
        sum_u_s, sum_b_s = 192_640_740, 18_536_549

        p_bar = sum_b_s / sum_u_s
        q_bar = sum_b_o / sum_u_o
        sum_e = sum_b_o - sum_u_o * p_bar
        sum_omega = sum_u_s * q_bar - sum_b_s
        ratio = sum_omega / sum_e

        assert p_bar == pytest.approx(0.096223410, abs=1e-8)
        assert q_bar == pytest.approx(0.219453325, abs=1e-8)
        assert ratio == pytest.approx(1.868, abs=1e-3)
        assert ratio != pytest.approx(1.552, abs=1e-2)  # never tune toward ADR171's 1.55 (F1)


# ---------------------------------------------------------------------------
# CSV writers — column shape + absence-cell emptiness on the real bytes.
# ---------------------------------------------------------------------------


class TestA2FileContents:
    def test_written_csv_has_expected_columns_and_empty_absence_cells(self, tmp_path: Path) -> None:
        by_race, crosswalk_rows = _fixture_pipeline_inputs()
        rows = nia.build_county_pole_rows(FIXTURE_UNIVERSE, by_race, crosswalk_rows)

        row_count, _ = nia.write_county_pole_artifact(rows, tmp_path / "a2.csv.gz")

        assert row_count == len(FIXTURE_UNIVERSE) * 4
        with gzip.open(tmp_path / "a2.csv.gz", "rt", encoding="utf-8", newline="") as handle:
            reader = csv.reader(handle)
            header = next(reader)
            body = list(reader)
        assert header == list(nia.A2_COLUMNS)
        assert len(body) == row_count
        c_row = next(r for r in body if r[0] == "01003" and r[1] == "C")
        assert c_row[3] == ""  # universe_u empty (ZERO_DENOMINATOR)
        assert c_row[5] == ""  # rate empty
        assert c_row[13] == "ZERO_DENOMINATOR"


class TestA3FileContents:
    def test_written_csv_has_expected_columns_and_pooled_row(self, tmp_path: Path) -> None:
        by_race, crosswalk_rows = _fixture_pipeline_inputs()
        rows = nia.build_reproduction_floor_rows(
            "artifact", FIXTURE_UNIVERSE, by_race, crosswalk_rows
        )

        row_count, _ = nia.write_reproduction_floor_artifact(rows, tmp_path / "a3.csv")

        assert row_count == 5
        with (tmp_path / "a3.csv").open(newline="", encoding="utf-8") as handle:
            reader = csv.reader(handle)
            header = next(reader)
            body = list(reader)
        assert header == list(nia.A3_COLUMNS)
        pooled_row = next(r for r in body if r[0] == nia.POOLED_POLE_LABEL)
        assert pooled_row[12] == "320"  # overlap_upper_bound
        individual_row = next(r for r in body if r[0] == "B")
        assert individual_row[11] == ""  # ratio_bribe_to_deprivation empty on individual rows
        assert individual_row[12] == ""  # overlap_upper_bound empty on individual rows


# ---------------------------------------------------------------------------
# Determinism — the double-run byte-identity gate (T4's own gate).
# ---------------------------------------------------------------------------


class TestDeterministicEmission:
    def test_a2_double_run_byte_identical(self, tmp_path: Path) -> None:
        """The real determinism gate: re-invoking the generator against the
        SAME canonical output path twice must produce byte-identical
        files. Deliberately writes to ONE shared path across both runs —
        ``_open_deterministic_gzip_text`` embeds the out path itself in the
        gzip header's FNAME field (``filename=str(path)``), so comparing
        two DIFFERENT paths' shas would fail on that field alone even with
        identical row content; that is not the property this gate tests.

        Builds the rows TWICE, independently, from two independent loads
        of the same fixture inputs — an earlier version of this test built
        ``rows`` once and wrote the SAME list object twice, which only
        proves the WRITE step deterministic (of course writing one object
        twice is byte-identical). This version exercises the full
        parse -> join -> classify -> pool -> assemble pipeline twice end
        to end, so a hash-seed-order regression in the BUILD step (see
        ``build_county_race_totals``'s ``sorted(keys)`` defense) would
        also be caught here."""
        by_race_1, crosswalk_rows_1 = _fixture_pipeline_inputs()
        by_race_2, crosswalk_rows_2 = _fixture_pipeline_inputs()
        rows1 = nia.build_county_pole_rows(FIXTURE_UNIVERSE, by_race_1, crosswalk_rows_1)
        rows2 = nia.build_county_pole_rows(FIXTURE_UNIVERSE, by_race_2, crosswalk_rows_2)
        out_path = tmp_path / "a2.csv.gz"

        _, sha1 = nia.write_county_pole_artifact(rows1, out_path)
        _, sha2 = nia.write_county_pole_artifact(rows2, out_path)

        assert sha1 == sha2

    def test_a2_row_order_independent_of_input_order(self, tmp_path: Path) -> None:
        by_race, crosswalk_rows = _fixture_pipeline_inputs()
        rows = nia.build_county_pole_rows(FIXTURE_UNIVERSE, by_race, crosswalk_rows)
        shuffled = tuple(reversed(rows))
        out_path = tmp_path / "a2.csv.gz"

        _, sha_ordered = nia.write_county_pole_artifact(rows, out_path)
        _, sha_shuffled = nia.write_county_pole_artifact(shuffled, out_path)

        assert sha_ordered == sha_shuffled

    def test_a3_double_run_byte_identical_across_all_three_variants(self, tmp_path: Path) -> None:
        by_race, crosswalk_rows = _fixture_pipeline_inputs()
        variant_fips = (
            ("artifact", FIXTURE_UNIVERSE),
            ("scopes", FIXTURE_UNIVERSE | {"46113"}),
            ("unrestricted", FIXTURE_UNIVERSE),
        )
        rows = []
        for name, fips in variant_fips:
            rows.extend(nia.build_reproduction_floor_rows(name, fips, by_race, crosswalk_rows))
        rows_tuple = tuple(rows)

        _, sha1 = nia.write_reproduction_floor_artifact(rows_tuple, tmp_path / "run1.csv")
        _, sha2 = nia.write_reproduction_floor_artifact(rows_tuple, tmp_path / "run2.csv")

        assert sha1 == sha2
        assert len(rows_tuple) == 15  # 3 variants x 5 rows each
