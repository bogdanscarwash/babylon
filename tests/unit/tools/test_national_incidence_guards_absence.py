"""Unit tests for the national-incidence artifact generator's three
absence + small-count guards (#334 Phase 0, T3b) —
``tools/make_national_incidence_artifact.py``.

T3b's scope (plan §3 guard register): **G3** (small-count damping — a
counting-statistics reliability MEASURE, never a stipulated sigmoid, ADR172
ruling 5), **G4** (ACS suppression policy — SUPPRESSED vs PRESENT vs
ROW_ABSENT, never imputed), **G5** (honest absence — reconciliation exact to
the declared universe size, Pine Ridge (``46102``) permanently
``DECLARED_HOLE``, never imputed from its retired predecessor ``46113``).

Each guard ships a ``TestMutation<G>`` class whose legs put the REAL module
function on the assertion path (standing rule: every sentinel/guard is
mutation-validated, ``tests/unit/sentinels/test_superstructure.py:5-7``) —
either (a) a baseline sub-test proving the real function passes a
discriminating correctness check, plus a neutering sub-test that
monkeypatches the real function to the named forbidden mutant and shows the
SAME correctness check reds (T3a's ``TestMutationG1``/``G2``/``G8``
precedent), or (b) a real violating input fed straight to the real function,
shown to raise (T3a's ``TestMutationG6``, T1's ``TestMutationG7`` precedent).
An arithmetic demonstration that never imports the module is NOT a mutation
leg — every leg below calls ``nia.<function>`` on its assertion path.

Guards G1/G2/G6/G8 are T3a's scope (``test_national_incidence_guards_arithmetic.py``);
G7 already shipped with T1. No ``[tool.mutmut]`` changes — ``tools/`` stays
out of mutmut's scope by design (same rationale as T3a's file header).
"""

from __future__ import annotations

import csv
import math
import sys
from pathlib import Path

import pytest

pytestmark = [pytest.mark.unit]

_REPO_ROOT = Path(__file__).resolve().parents[3]
_TOOLS_DIR = _REPO_ROOT / "tools"
_FIXTURES_DIR = _REPO_ROOT / "tests/fixtures/national_incidence"
sys.path.insert(0, str(_TOOLS_DIR))

import make_national_incidence_artifact as nia  # type: ignore[import-not-found]  # noqa: E402


def _load_csv_rows(name: str) -> list[dict[str, str]]:
    path = _FIXTURES_DIR / name
    with path.open(newline="", encoding="utf-8") as handle:
        return list(csv.DictReader(handle))


#: The three named small-count fixtures the plan's G3 row requires
#: (proposal lines 181-184): Loving County TX, Elliott County KY, King
#: County TX — all three famously tiny-population counties whose
#: poverty-universe counts are small enough that a raw rate deviation is
#: dominated by sampling noise, not real signal.
_SATURATING_FIPS = frozenset({"48301", "21063", "48269"})  # Loving, Elliott, King


# ---------------------------------------------------------------------------
# G3 — small-count damping: sigma_damped = |w| * damp(u); damp monotone
# non-decreasing, damp(0) undefined, damp -> 1 as u -> infinity.
# ---------------------------------------------------------------------------


class TestDamp:
    """``damp(u)``'s own correctness contract, independent of any fixture
    ranking: undefined at u<=0 (G2's territory), the exact closed form
    (1 - 1/sqrt(u)), and the asymptotic-to-1 behavior the law names."""

    def test_damp_raises_on_zero(self) -> None:
        with pytest.raises(nia.ArtifactGenerationError, match="undefined"):
            nia.damp(0)

    def test_damp_raises_on_negative(self) -> None:
        with pytest.raises(nia.ArtifactGenerationError, match="undefined"):
            nia.damp(-1)

    def test_damp_matches_the_derived_closed_form(self) -> None:
        for u in (1, 15, 31, 78, 1000):
            assert nia.damp(u) == pytest.approx(1.0 - 1.0 / math.sqrt(u))

    def test_damp_is_strictly_increasing_across_the_named_fixture_universes(self) -> None:
        assert nia.damp(15) < nia.damp(31) < nia.damp(78) < nia.damp(1_000_000)

    def test_damp_approaches_one_for_a_very_large_universe(self) -> None:
        assert nia.damp(1_000_000_000) == pytest.approx(1.0, abs=1e-4)
        assert nia.damp(1_000_000_000) < 1.0


class TestComputeDampedSigma:
    """``sigma_damped = |w| * damp(u)``, with ``damping_weight`` published
    separately (never baked silently into ``sigma_damped`` alone — T3b
    step 2's auditability requirement)."""

    def test_publishes_damping_weight_equal_to_damp_of_u(self) -> None:
        result = nia.compute_damped_sigma(w=0.5, universe_u=31)

        assert result.damping_weight == pytest.approx(nia.damp(31))
        assert result.sigma_damped == pytest.approx(0.5 * nia.damp(31))

    def test_uses_the_absolute_value_of_w(self) -> None:
        positive = nia.compute_damped_sigma(w=0.4, universe_u=78)
        negative = nia.compute_damped_sigma(w=-0.4, universe_u=78)

        assert positive.sigma_damped == pytest.approx(negative.sigma_damped)
        assert positive.damping_weight == pytest.approx(negative.damping_weight)


class TestSmallCountDampingDemotesTheNamedFixtures:
    """``fixtures/g3_small_count_damping_cells.csv``: Loving TX (u=15),
    Elliott KY (u=31), King TX (u=78) carry the three highest raw ``|w|``
    in the fixture set (they "saturate |w| undamped" — the plan's G3 row) —
    undamped they occupy the top 3 of a 7-county ranking. Four large-``u``
    control counties carry slightly lower raw ``|w|`` but damp to
    essentially 1.0, and the DoD requires the real guard to demote all
    three named counties out of the damped top 3."""

    def _rows(self) -> list[dict[str, str]]:
        return _load_csv_rows("g3_small_count_damping_cells.csv")

    def _undamped_ranking(self) -> list[str]:
        rows = self._rows()
        return sorted((r["fips"] for r in rows), key=lambda f: -self._w_by_fips()[f])

    def _w_by_fips(self) -> dict[str, float]:
        return {r["fips"]: float(r["w"]) for r in self._rows()}

    def test_undamped_top_three_is_exactly_the_three_named_counties(self) -> None:
        """Sanity check on the fixture itself (damp=1, no module call):
        proves the three named counties really do saturate |w| in this
        fixture before any guard is exercised."""
        top3 = set(self._undamped_ranking()[:3])

        assert top3 == _SATURATING_FIPS

    def test_real_damping_demotes_all_three_out_of_the_damped_top_three(self) -> None:
        rows = self._rows()
        damped_sigma = {
            r["fips"]: nia.compute_damped_sigma(
                w=float(r["w"]), universe_u=int(r["universe_u"])
            ).sigma_damped
            for r in rows
        }
        top3 = set(sorted(damped_sigma, key=lambda f: -damped_sigma[f])[:3])

        assert top3.isdisjoint(_SATURATING_FIPS), (
            f"the three saturating counties must be demoted out of the damped top 3, "
            f"got top3={top3}"
        )


class TestMutationG3:
    """G3's two named red legs: (a) ``damp ≡ 1`` puts the three named
    counties back at the top of the ranking; (b) a non-monotone ``damp``
    reds the monotonicity check. Both monkeypatch ``nia.damp`` directly —
    ``compute_damped_sigma`` looks up ``damp`` as a module global at call
    time, so patching ``nia.damp`` reaches it without any extra wiring."""

    def _rows(self) -> list[dict[str, str]]:
        return _load_csv_rows("g3_small_count_damping_cells.csv")

    def _damped_top3(self) -> set[str]:
        rows = self._rows()
        damped_sigma = {
            r["fips"]: nia.compute_damped_sigma(
                w=float(r["w"]), universe_u=int(r["universe_u"])
            ).sigma_damped
            for r in rows
        }
        return set(sorted(damped_sigma, key=lambda f: -damped_sigma[f])[:3])

    def test_the_real_damp_demotes_the_three_saturating_counties(self) -> None:
        """Baseline: proves the leg below isn't vacuous."""
        assert self._damped_top3().isdisjoint(_SATURATING_FIPS)

    def test_neutering_damp_to_constant_one_reds_the_demotion_check(
        self, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        def constant_one_mutant(universe_u: int) -> float:
            return 1.0

        monkeypatch.setattr(nia, "damp", constant_one_mutant)

        with pytest.raises(AssertionError):
            assert self._damped_top3().isdisjoint(_SATURATING_FIPS)

    def test_the_real_damp_is_monotone_on_the_named_fixture_universes(self) -> None:
        """Baseline: proves the leg below isn't vacuous."""
        assert nia.damp(31) <= nia.damp(78)

    def test_neutering_damp_to_a_non_monotone_mutant_reds_the_monotonicity_check(
        self, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        def non_monotone_mutant(universe_u: int) -> float:
            # Rises then falls across exactly the named fixtures' universes
            # (31 -> 0.31, 78 -> 0.22): a real violation of "monotone
            # non-decreasing in u" on the same u values G3's own fixtures use.
            if universe_u <= 50:
                return universe_u / 100.0
            return (100.0 - universe_u) / 100.0

        monkeypatch.setattr(nia, "damp", non_monotone_mutant)

        with pytest.raises(AssertionError):
            assert nia.damp(31) <= nia.damp(78)


# ---------------------------------------------------------------------------
# G4 — ACS suppression policy: SUPPRESSED vs PRESENT by a declared,
# published rule; a missing row is ROW_ABSENT, never imputed.
# ---------------------------------------------------------------------------


class TestClassifySuppression:
    """``fixtures/g4_suppression_classification_cells.csv``: 99201
    (u=5, b=0) is a plausible genuine zero (PRESENT); 99202 (u=100, b=0) is
    statistically implausible as a genuine zero under the declared
    reference rate (SUPPRESSED)."""

    def _cells(self) -> dict[str, dict[str, str]]:
        return {r["fips"]: r for r in _load_csv_rows("g4_suppression_classification_cells.csv")}

    def test_small_universe_zero_is_present(self) -> None:
        row = self._cells()["99201"]

        result = nia.classify_suppression(
            universe_u=int(row["universe_u"]), below_b=int(row["below_b"])
        )

        assert result == "PRESENT"

    def test_large_universe_zero_is_suppressed(self) -> None:
        row = self._cells()["99202"]

        result = nia.classify_suppression(
            universe_u=int(row["universe_u"]), below_b=int(row["below_b"])
        )

        assert result == "SUPPRESSED"

    def test_raises_when_universe_is_not_positive(self) -> None:
        with pytest.raises(nia.ArtifactGenerationError, match="G2"):
            nia.classify_suppression(universe_u=0, below_b=0)

    def test_raises_when_below_is_not_zero(self) -> None:
        with pytest.raises(nia.ArtifactGenerationError, match="below_b"):
            nia.classify_suppression(universe_u=500, below_b=20)

    def test_boundary_u28_is_present_u29_is_suppressed(self) -> None:
        """T4 review obligation (freebie): the exact threshold where
        ``(1 - SUPPRESSION_REFERENCE_RATE) ** u`` crosses
        ``SUPPRESSION_IMPLAUSIBILITY_ALPHA`` (0.9**28 ≈ 0.05233 >= 0.05;
        0.9**29 ≈ 0.04710 < 0.05) — pins the classifier's actual decision
        boundary, not just interior examples."""
        assert nia.classify_suppression(universe_u=28, below_b=0) == "PRESENT"
        assert nia.classify_suppression(universe_u=29, below_b=0) == "SUPPRESSED"


class TestClassifyAbsence:
    """The full per-(fips, pole) absence classification: ROW_ABSENT
    (missing cell), ZERO_DENOMINATOR (G2 reused), SUPPRESSED/PRESENT (G4),
    DECLARED_HOLE (A1, overrides everything else for a declared-hole
    fips)."""

    _DECLARED_HOLE_FIPS = frozenset({"46102", "02158"})

    def test_normal_present_cell(self) -> None:
        row = {r["fips"]: r for r in _load_csv_rows("g4_suppression_classification_cells.csv")}[
            "99203"
        ]
        cell = nia.PoleCellPair(universe_u=int(row["universe_u"]), below_b=int(row["below_b"]))

        result = nia.classify_absence(
            cell, fips="99203", declared_hole_fips=self._DECLARED_HOLE_FIPS
        )

        assert result.absence_class == "PRESENT"
        assert result.rate == pytest.approx(20 / 500)

    def test_zero_denominator_cell_defers_to_g2(self) -> None:
        cell = nia.PoleCellPair(universe_u=0, below_b=0)

        result = nia.classify_absence(
            cell, fips="99999", declared_hole_fips=self._DECLARED_HOLE_FIPS
        )

        assert result.absence_class == "ZERO_DENOMINATOR"
        assert result.rate is None

    def test_missing_row_is_row_absent(self) -> None:
        result = nia.classify_absence(
            None, fips="99999", declared_hole_fips=self._DECLARED_HOLE_FIPS
        )

        assert result.absence_class == "ROW_ABSENT"
        assert result.rate is None

    def test_suppressed_cell_carries_an_empty_rate(self) -> None:
        cell = nia.PoleCellPair(universe_u=100, below_b=0)

        result = nia.classify_absence(
            cell, fips="99202", declared_hole_fips=self._DECLARED_HOLE_FIPS
        )

        assert result.absence_class == "SUPPRESSED"
        assert result.rate is None  # never a fabricated 0.0 — same law as G2

    def test_declared_hole_fips_overrides_any_cell_data(self) -> None:
        """Even if a (malformed) cell somehow carried data for a
        declared-hole fips, DECLARED_HOLE must win — the whole point of
        the classification is that this data is never trusted."""
        cell = nia.PoleCellPair(universe_u=50, below_b=5)

        result = nia.classify_absence(
            cell, fips="46102", declared_hole_fips=self._DECLARED_HOLE_FIPS
        )

        assert result.absence_class == "DECLARED_HOLE"
        assert result.rate is None

    def test_pine_ridge_missing_row_classifies_declared_hole_not_row_absent(self) -> None:
        result = nia.classify_absence(
            None, fips="46102", declared_hole_fips=self._DECLARED_HOLE_FIPS
        )

        assert result.absence_class == "DECLARED_HOLE"


class TestMutationG4:
    """The red leg the plan's guard table names for G4: neuter the REAL
    ``nia.classify_suppression`` so it no longer distinguishes SUPPRESSED
    from PRESENT (collapses the two classes), and show the disclosure
    correctness check reds when re-run through the same public name."""

    def test_the_real_classifier_distinguishes_suppressed_from_present(self) -> None:
        """Baseline: proves the leg below isn't vacuous — the same
        declared rule, applied to a small (plausible) and a large
        (implausible) zero-count universe, must land in different classes."""
        small_universe = nia.classify_suppression(universe_u=5, below_b=0)
        large_universe = nia.classify_suppression(universe_u=100, below_b=0)

        assert small_universe != large_universe
        assert {small_universe, large_universe} == {"PRESENT", "SUPPRESSED"}

    def test_neutering_the_classifier_to_always_present_reds_the_distinction_check(
        self, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        def always_present_mutant(universe_u: int, below_b: int) -> str:
            # the forbidden mutation: collapses SUPPRESSED into PRESENT
            return "PRESENT"

        monkeypatch.setattr(nia, "classify_suppression", always_present_mutant)

        small_universe = nia.classify_suppression(universe_u=5, below_b=0)
        large_universe = nia.classify_suppression(universe_u=100, below_b=0)
        with pytest.raises(AssertionError):
            assert small_universe != large_universe


# ---------------------------------------------------------------------------
# G5 — honest absence: present + absent == universe_size, exact; Pine
# Ridge (46102) is DECLARED_HOLE in every universe variant, never imputed
# from its retired predecessor 46113.
# ---------------------------------------------------------------------------


class TestReconcileAbsenceCounts:
    """``fixtures/g5_absence_reconciliation_cells.csv``: 20 (fips, pole)
    cells — 12 PRESENT, 3 ZERO_DENOMINATOR, 2 ROW_ABSENT, 1 DECLARED_HOLE
    (46102, Pine Ridge), 2 SUPPRESSED. counties_present=12,
    counties_absent=8, universe_size=20 (exact)."""

    def _labels(self) -> list[str]:
        return [r["absence_class"] for r in _load_csv_rows("g5_absence_reconciliation_cells.csv")]

    def test_reconciles_exactly_to_the_declared_universe_size(self) -> None:
        result = nia.reconcile_absence_counts(self._labels(), universe_size=20)

        assert result.counties_present == 12
        assert result.counties_absent == 8
        assert result.counties_present + result.counties_absent == 20
        assert result.counts_by_class == {
            "PRESENT": 12,
            "ZERO_DENOMINATOR": 3,
            "ROW_ABSENT": 2,
            "DECLARED_HOLE": 1,
            "SUPPRESSED": 2,
        }

    def test_raises_on_mismatched_universe_size(self) -> None:
        with pytest.raises(nia.ArtifactGenerationError, match="universe_size"):
            nia.reconcile_absence_counts(self._labels(), universe_size=21)

    def test_raises_on_unknown_absence_class(self) -> None:
        with pytest.raises(nia.ArtifactGenerationError, match="unknown"):
            nia.reconcile_absence_counts(["PRESENT", "MADE_UP_CLASS"], universe_size=2)


class TestMutationG5:
    """The red leg the plan's guard table names for G5: drop one absence
    class from the reconciliation's counting, and show the correctness
    check reds."""

    def _labels(self) -> list[str]:
        return [r["absence_class"] for r in _load_csv_rows("g5_absence_reconciliation_cells.csv")]

    def test_the_real_reconciliation_counts_all_five_classes(self) -> None:
        """Baseline: proves the leg below isn't vacuous."""
        result = nia.reconcile_absence_counts(self._labels(), universe_size=20)

        assert result.counties_absent == 8

    def test_dropping_suppressed_from_the_reconciliation_reds(
        self, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        def drops_suppressed_mutant(
            absence_classes: list[str], *, universe_size: int
        ) -> nia.AbsenceReconciliation:
            # the forbidden mutation: SUPPRESSED silently never counted as absent
            counts = dict.fromkeys(nia.ABSENCE_CLASSES, 0)
            for label in absence_classes:
                counts[label] += 1
            counties_present = counts["PRESENT"]
            counties_absent = sum(
                counts[label]
                for label in nia.ABSENCE_CLASSES
                if label not in ("PRESENT", "SUPPRESSED")
            )
            return nia.AbsenceReconciliation(
                counts_by_class=counts,
                counties_present=counties_present,
                counties_absent=counties_absent,
                universe_size=universe_size,
            )

        monkeypatch.setattr(nia, "reconcile_absence_counts", drops_suppressed_mutant)

        result = nia.reconcile_absence_counts(self._labels(), universe_size=20)
        with pytest.raises(AssertionError):
            assert result.counties_absent == 8


class TestPineRidgeNeverImputed:
    """G5's Pine Ridge leg: ``46102`` (Oglala Lakota) must classify
    ``DECLARED_HOLE`` in every universe variant and must never be
    attributed to data pulled under its retired predecessor ``46113``
    (Shannon County, rows only 2010-2014, stale before the pinned 2019
    vintage)."""

    def test_pine_ridge_with_no_source_and_declared_hole_passes(self) -> None:
        """Baseline: proves the legs below aren't vacuous."""
        nia.assert_no_pine_ridge_imputation("46102", "DECLARED_HOLE")  # no raise == pass

    def test_pine_ridge_imputed_from_retired_predecessor_raises(self) -> None:
        # A naive check that only looks at absence_class (ignoring
        # source_fips) would wrongly accept this — proving the real guard
        # catches something a shallower check would miss.
        naive_mutant_would_pass = "DECLARED_HOLE" == "DECLARED_HOLE"  # noqa: PLR0133
        assert naive_mutant_would_pass is True

        with pytest.raises(nia.ArtifactGenerationError, match="46113"):
            nia.assert_no_pine_ridge_imputation("46102", "DECLARED_HOLE", source_fips="46113")

    def test_pine_ridge_classified_as_anything_but_declared_hole_raises(self) -> None:
        with pytest.raises(nia.ArtifactGenerationError, match="DECLARED_HOLE"):
            nia.assert_no_pine_ridge_imputation("46102", "PRESENT")

    def test_non_pine_ridge_fips_is_unaffected(self) -> None:
        # A normal county classified PRESENT, or even one whose
        # source_fips happens to equal 46113's string, is out of scope —
        # this guard is Pine-Ridge-specific.
        nia.assert_no_pine_ridge_imputation("48301", "PRESENT")  # no raise == pass
        nia.assert_no_pine_ridge_imputation("48301", "PRESENT", source_fips="46113")  # no raise
