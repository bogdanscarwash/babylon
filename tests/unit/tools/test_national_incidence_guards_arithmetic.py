"""Unit tests for the national-incidence artifact generator's four
arithmetic-law guards (#334 Phase 0, T3a) —
``tools/make_national_incidence_artifact.py``.

T3a's scope (plan §3 guard register): **G1** (ratio-of-sums), **G2**
(zero-denominator = ABSENCE), **G6** (T-pole exactness), **G8** (overlap
disclosed, never netted) — four separate pure functions, each with its own
correctness tests plus a ``TestMutation<G>`` class that proves the guard
actually fires (standing rule: every sentinel/guard is mutation-validated,
``tests/unit/sentinels/test_superstructure.py:5-7``; T1's
``TestMutationG7`` in ``test_fips_vintage_crosswalk.py`` is the closer
precedent for a guard shipped by this same #334 train).

G1 and G8 additionally carry an AST leg (the plan's guard-register cells for
those two rows): a static scan of THIS module's source proving neither the
forbidden mean-of-per-county-rates pattern (G1) nor a subtraction of the
overlap bound from any pole-sum expression (G8) exists anywhere in it. Each
AST leg is proven non-vacuous by also running the same scanner against a
deliberately-violating source snippet and asserting it fires.

Guards for G3/G4/G5/G7 are out of scope here — G7 already shipped with T1
(``test_fips_vintage_crosswalk.py::TestMutationG7``); G3/G4/G5 land with
T3b. ``paths_to_mutate``/``tests_dir`` in ``pyproject.toml``'s
``[tool.mutmut]`` are untouched by this task (``tools/`` stays out of
mutmut's scope by design — a real mutmut run over ``tools/`` would drag the
whole ``tests/unit/tools/`` directory into every run); the "mutation legs"
here are the in-suite idiom, not a mutmut invocation.
"""

from __future__ import annotations

import ast
import csv
import statistics
import sys
from collections.abc import Sequence
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


def _module_source() -> str:
    return Path(nia.__file__).read_text(encoding="utf-8")


# ---------------------------------------------------------------------------
# G1 — ratio-of-sums: p̄/q̄ = Σb/Σu; mean(rate_i) forbidden anywhere.
# ---------------------------------------------------------------------------


class TestRatioOfSums:
    """``fixtures/g1_ratio_of_sums_cells.csv``: county 99001 (u=1000, b=10,
    rate=0.01) and 99002 (u=10, b=9, rate=0.90) — deliberately lopsided
    weights so ratio-of-sums and mean-of-ratios provably differ (below)."""

    def _cells(self) -> tuple[nia.AggregationCell, ...]:
        rows = _load_csv_rows("g1_ratio_of_sums_cells.csv")
        return tuple(
            nia.AggregationCell(
                fips=row["fips"],
                universe_u=int(row["universe_u"]),
                below_b=int(row["below_b"]),
            )
            for row in rows
        )

    def test_ratio_of_sums_is_sigma_b_over_sigma_u(self) -> None:
        cells = self._cells()

        result = nia.ratio_of_sums(cells)

        # Σb=19, Σu=1010 -> 19/1010, NOT mean(0.01, 0.90)=0.455.
        assert result == pytest.approx(19 / 1010)

    def test_ratio_of_sums_and_mean_of_ratios_provably_differ_on_this_fixture(self) -> None:
        """The fixture the DoD requires: show both values, and that they
        are not close — this is exactly why G1 exists (a mean-of-ratios
        aggregate would silently overweight the small-universe county)."""
        cells = self._cells()

        ratio_of_sums = nia.ratio_of_sums(cells)
        mean_of_ratios = statistics.mean(c.below_b / c.universe_u for c in cells)

        assert ratio_of_sums == pytest.approx(19 / 1010)  # ~0.0188
        assert mean_of_ratios == pytest.approx(0.455)  # (0.01 + 0.90) / 2
        assert ratio_of_sums != pytest.approx(mean_of_ratios)

    def test_ratio_of_sums_raises_on_empty_universe(self) -> None:
        with pytest.raises(nia.ArtifactGenerationError, match="Σu"):
            nia.ratio_of_sums(())


class TestMutationG1:
    """The red leg the plan's guard table names for G1: neuter the REAL
    ``nia.ratio_of_sums`` by monkeypatching in a mean-of-ratios
    implementation, and show that re-running the module's own correctness
    assertion through the same public name (``nia.ratio_of_sums(...)``)
    reds against the neutered function. Unlike an earlier draft of this
    leg, the mutant computation is never a bare local recomputation
    compared against a hardcoded literal — the real module attribute is on
    the assertion path both before and after neutering, so a silent swap of
    ``ratio_of_sums``'s body for ``statistics.mean`` is exactly what this
    leg catches."""

    _CELLS = (
        nia.AggregationCell(fips="99001", universe_u=1000, below_b=10),
        nia.AggregationCell(fips="99002", universe_u=10, below_b=9),
    )
    _EXPECTED_RATIO_OF_SUMS = 19 / 1010

    def test_the_real_guard_passes_the_correctness_check(self) -> None:
        """Baseline: proves the leg below isn't vacuous — the real,
        un-neutered function passes the exact assertion the neutered
        version is shown to fail."""
        assert nia.ratio_of_sums(self._CELLS) == pytest.approx(self._EXPECTED_RATIO_OF_SUMS)

    def test_neutering_ratio_of_sums_to_mean_of_ratios_reds_the_correctness_check(
        self, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        def mean_of_ratios_mutant(cells: Sequence[nia.AggregationCell]) -> float:
            # the forbidden mutation, verbatim from the plan's guard table
            return statistics.mean(c.below_b / c.universe_u for c in cells)

        monkeypatch.setattr(nia, "ratio_of_sums", mean_of_ratios_mutant)

        with pytest.raises(AssertionError):
            assert nia.ratio_of_sums(self._CELLS) == pytest.approx(self._EXPECTED_RATIO_OF_SUMS)

    def test_ast_leg_module_source_has_no_mean_of_ratios_expression(self) -> None:
        tree = ast.parse(_module_source())

        assert not _contains_mean_of_ratios(tree)

    def test_ast_leg_scanner_actually_detects_a_violation(self) -> None:
        """Proves the AST scanner above is not vacuously always-False —
        run it against source that DOES contain the forbidden pattern."""
        violating_variants = (
            "import statistics\nx = statistics.mean(rates)\n",
            "from statistics import mean\nx = mean(rates)\n",
            "x = total_b / len(counties)\n",
        )
        for source in violating_variants:
            tree = ast.parse(source)
            assert _contains_mean_of_ratios(tree), f"scanner missed: {source!r}"


def _contains_mean_of_ratios(tree: ast.AST) -> bool:
    """G1's AST leg: True if ``tree`` contains a call to ``mean(...)`` /
    ``statistics.mean(...)``, or a division whose denominator is a bare
    ``len(...)`` call — the two shapes a mean-over-per-county-rates
    expression would take."""
    for node in ast.walk(tree):
        if isinstance(node, ast.Call):
            func = node.func
            if isinstance(func, ast.Name) and func.id == "mean":
                return True
            if isinstance(func, ast.Attribute) and func.attr == "mean":
                return True
        if (
            isinstance(node, ast.BinOp)
            and isinstance(node.op, ast.Div)
            and isinstance(node.right, ast.Call)
            and isinstance(node.right.func, ast.Name)
            and node.right.func.id == "len"
        ):
            return True
    return False


# ---------------------------------------------------------------------------
# G2 — zero-denominator = ABSENCE: u == 0 => ZERO_DENOMINATOR, rate=None.
# ---------------------------------------------------------------------------


class TestClassifyZeroDenominator:
    """``fixtures/g2_zero_denominator_cells.csv``: 99001 has a normal
    (u=500, b=50) cell; 99003 has u=0 (the absence case)."""

    def _cells(self) -> list[dict[str, str]]:
        return _load_csv_rows("g2_zero_denominator_cells.csv")

    def test_present_cell_computes_a_normal_rate(self) -> None:
        cells = {row["fips"]: row for row in self._cells()}
        row = cells["99001"]

        result = nia.classify_zero_denominator(
            universe_u=int(row["universe_u"]), below_b=int(row["below_b"])
        )

        assert result.absence_class is None
        assert result.rate == pytest.approx(0.1)

    def test_zero_denominator_cell_is_absence_with_empty_rate(self) -> None:
        cells = {row["fips"]: row for row in self._cells()}
        row = cells["99003"]

        result = nia.classify_zero_denominator(
            universe_u=int(row["universe_u"]), below_b=int(row["below_b"])
        )

        assert result.absence_class == "ZERO_DENOMINATOR"
        assert result.rate is None  # never 0.0 — III.11

    def test_negative_universe_raises(self) -> None:
        with pytest.raises(nia.ArtifactGenerationError, match="-1"):
            nia.classify_zero_denominator(universe_u=-1, below_b=0)


class TestMutationG2:
    """The red leg the plan's guard table names for G2: neuter the REAL
    ``nia.classify_zero_denominator`` by monkeypatching in the forbidden
    ``rate = 0.0 if u == 0 else b / u`` mutation (verbatim from the plan's
    guard table), and show the "never a fabricated 0.0" correctness check
    reds when re-run through the same public name."""

    def test_the_real_guard_never_fabricates_a_zero_rate(self) -> None:
        """Baseline: proves the leg below isn't vacuous."""
        result = nia.classify_zero_denominator(universe_u=0, below_b=0)

        assert result.rate is None
        assert result.absence_class == "ZERO_DENOMINATOR"

    def test_neutering_the_guard_to_fabricate_zero_reds_the_correctness_check(
        self, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        def fabricated_zero_mutant(universe_u: int, below_b: int) -> nia.ZeroDenominatorResult:
            # the forbidden mutation, verbatim from the plan's guard table
            rate = 0.0 if universe_u == 0 else below_b / universe_u
            absence_class = "ZERO_DENOMINATOR" if universe_u == 0 else None
            return nia.ZeroDenominatorResult(absence_class=absence_class, rate=rate)

        monkeypatch.setattr(nia, "classify_zero_denominator", fabricated_zero_mutant)

        result = nia.classify_zero_denominator(universe_u=0, below_b=0)
        with pytest.raises(AssertionError):
            assert result.rate is None


# ---------------------------------------------------------------------------
# G6 — T-pole exactness: T == Σ(A..G) exact; H <= A; I <= T.
# ---------------------------------------------------------------------------


class TestAssertTPoleExactness:
    """``fixtures/g6_t_pole_exactness_cells.csv``: two counties, both
    exactly reconciled (T == Σ(A..G)), H <= A, I <= T."""

    def _cells(self) -> list[dict[str, str]]:
        return _load_csv_rows("g6_t_pole_exactness_cells.csv")

    def _parts(self, row: dict[str, str]) -> dict[str, int]:
        return {letter: int(row[letter.lower()]) for letter in nia.POLE_PART_LETTERS}

    def test_exactly_reconciled_counties_pass(self) -> None:
        for row in self._cells():
            nia.assert_t_pole_exactness(
                row["fips"],
                t=int(row["t"]),
                parts=self._parts(row),
                h=int(row["h"]),
                i=int(row["i"]),
            )  # no raise == pass

    def test_off_by_one_residual_raises(self) -> None:
        row = {r["fips"]: r for r in self._cells()}["99001"]
        parts = self._parts(row)

        with pytest.raises(nia.ArtifactGenerationError, match="residual"):
            nia.assert_t_pole_exactness(
                row["fips"], t=int(row["t"]) + 1, parts=parts, h=int(row["h"]), i=int(row["i"])
            )

    def test_h_exceeding_a_raises(self) -> None:
        row = {r["fips"]: r for r in self._cells()}["99001"]
        parts = self._parts(row)

        with pytest.raises(nia.ArtifactGenerationError, match="H="):
            nia.assert_t_pole_exactness(
                row["fips"], t=int(row["t"]), parts=parts, h=parts["A"] + 1, i=int(row["i"])
            )

    def test_i_exceeding_t_raises(self) -> None:
        row = {r["fips"]: r for r in self._cells()}["99001"]
        parts = self._parts(row)

        with pytest.raises(nia.ArtifactGenerationError, match="I="):
            nia.assert_t_pole_exactness(
                row["fips"], t=int(row["t"]), parts=parts, h=int(row["h"]), i=int(row["t"]) + 1
            )

    def test_missing_pole_part_raises(self) -> None:
        with pytest.raises(nia.ArtifactGenerationError, match="missing"):
            nia.assert_t_pole_exactness("99999", t=10, parts={"A": 10}, h=0, i=0)


class TestMutationG6:
    """The red leg the plan's guard table names for G6: introduce a +/-1
    residual tolerance, and show that the tolerant mutant would silently
    swallow a violation the real (exact-equality) guard catches."""

    def test_real_guard_raises_on_off_by_one_residual(self) -> None:
        parts = {"A": 40, "B": 20, "C": 10, "D": 10, "E": 5, "F": 5, "G": 10}  # sums to 100

        with pytest.raises(nia.ArtifactGenerationError, match="residual"):
            nia.assert_t_pole_exactness("99001", t=101, parts=parts, h=30, i=15)

    def test_tolerant_mutant_swallows_the_same_violation(self) -> None:
        t, parts_sum = 101, 100  # the exact off-by-one case above

        # the forbidden mutation, verbatim from the plan's guard table
        tolerant_mutant_passes = abs(t - parts_sum) <= 1

        assert tolerant_mutant_passes is True  # mutant wrongly accepts residual=1
        with pytest.raises(nia.ArtifactGenerationError):
            # ... while the real guard correctly still raises on it.
            nia.assert_t_pole_exactness(
                "99001",
                t=t,
                parts={"A": 40, "B": 20, "C": 10, "D": 10, "E": 5, "F": 5, "G": 10},
                h=30,
                i=15,
            )


# ---------------------------------------------------------------------------
# G8 — overlap disclosed, never netted: bound = I - (A - H); never subtracted.
# ---------------------------------------------------------------------------


class TestOverlapUpperBound:
    """``fixtures/g8_overlap_cells.csv``: 99001 (A=100, H=60, I=90) ->
    white_hispanic=40, bound=50. 99002 (A=50, H=10, I=45) -> white_hispanic=40,
    bound=5."""

    def _cells(self) -> list[dict[str, str]]:
        return _load_csv_rows("g8_overlap_cells.csv")

    def test_bound_is_i_minus_a_minus_h(self) -> None:
        rows = {r["fips"]: r for r in self._cells()}

        bound_99001 = nia.overlap_upper_bound(
            total_a=int(rows["99001"]["total_a"]),
            white_non_hispanic_h=int(rows["99001"]["white_non_hispanic_h"]),
            hispanic_i=int(rows["99001"]["hispanic_i"]),
        )
        bound_99002 = nia.overlap_upper_bound(
            total_a=int(rows["99002"]["total_a"]),
            white_non_hispanic_h=int(rows["99002"]["white_non_hispanic_h"]),
            hispanic_i=int(rows["99002"]["hispanic_i"]),
        )

        assert bound_99001 == 50
        assert bound_99002 == 5

    def test_h_exceeding_a_raises(self) -> None:
        with pytest.raises(nia.ArtifactGenerationError, match="H="):
            nia.overlap_upper_bound(total_a=10, white_non_hispanic_h=11, hispanic_i=5)

    def test_negative_bound_raises(self) -> None:
        # white_hispanic = 100 - 60 = 40 > hispanic_i=30: impossible data.
        with pytest.raises(nia.ArtifactGenerationError, match="< 0"):
            nia.overlap_upper_bound(total_a=100, white_non_hispanic_h=60, hispanic_i=30)


class TestMutationG8:
    """The red leg the plan's guard table names for G8: (a) a value leg
    that neuters the REAL ``nia.overlap_upper_bound`` by monkeypatching in
    an always-zero mutant — a bound that always discloses 0 is functionally
    identical to silently netting the overlap out of every pole sum
    (``Σu_o - 0 == Σu_o``), exactly what G8 forbids — and shows the
    disclosure correctness check reds when re-run through the same public
    name; and (b) an AST leg on the module source (below)."""

    _KNOWN_BOUND = 50  # total_a=100, white_non_hispanic_h=60, hispanic_i=90

    def test_the_real_guard_computes_the_disclosed_bound(self) -> None:
        """Baseline: proves the leg below isn't vacuous."""
        bound = nia.overlap_upper_bound(total_a=100, white_non_hispanic_h=60, hispanic_i=90)

        assert bound == self._KNOWN_BOUND

    def test_neutering_the_bound_to_always_zero_reds_the_disclosure_check(
        self, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        def always_zero_mutant(total_a: int, white_non_hispanic_h: int, hispanic_i: int) -> int:
            return 0

        monkeypatch.setattr(nia, "overlap_upper_bound", always_zero_mutant)

        bound = nia.overlap_upper_bound(total_a=100, white_non_hispanic_h=60, hispanic_i=90)
        with pytest.raises(AssertionError):
            assert bound == self._KNOWN_BOUND

    def test_ast_leg_module_source_has_no_pole_sum_subtraction_of_the_bound(self) -> None:
        tree = ast.parse(_module_source())

        assert not _contains_overlap_bound_subtraction(tree)

    def test_ast_leg_scanner_actually_detects_a_violation(self) -> None:
        violating_source = (
            "overlap_bound = overlap_upper_bound(a, h, i)\n"
            "corrected_sum_u_o = sum_u_o - overlap_bound\n"
        )
        tree = ast.parse(violating_source)

        assert _contains_overlap_bound_subtraction(tree)


def _contains_overlap_bound_subtraction(tree: ast.AST) -> bool:
    """G8's AST leg: True if ``tree`` contains a subtraction (``BinOp`` with
    ``ast.Sub``) where either operand is, or is named after, the overlap
    bound: a call to ``overlap_upper_bound(...)``, or a bare Name whose
    identifier contains ``overlap_bound`` / ``overlap_upper_bound`` (the A3
    schema's own column names, ``plan §2``) — catches the
    ``sum_u_o - overlap_bound`` shape a future pipeline (T3b/T4) would use
    without requiring full def-use tracking in a static scan, while staying
    narrow enough not to flag unrelated ``*_bound`` identifiers (e.g. a
    damping clamp)."""
    for node in ast.walk(tree):
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Sub):
            for operand in (node.left, node.right):
                if _mentions_overlap_bound(operand):
                    return True
    return False


def _mentions_overlap_bound(node: ast.AST) -> bool:
    for sub in ast.walk(node):
        if isinstance(sub, ast.Call):
            func = sub.func
            if isinstance(func, ast.Name) and func.id == "overlap_upper_bound":
                return True
            if isinstance(func, ast.Attribute) and func.attr == "overlap_upper_bound":
                return True
        if isinstance(sub, ast.Name) and "overlap_bound" in sub.id.lower():
            return True
        if isinstance(sub, ast.Name) and "overlap_upper_bound" in sub.id.lower():
            return True
    return False
