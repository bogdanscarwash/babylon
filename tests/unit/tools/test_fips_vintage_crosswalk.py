"""Unit tests for the FIPS vintage crosswalk artifact generator (#334 Phase 0,
T1) — ``tools/make_fips_vintage_crosswalk.py``.

Pure-logic tests: no drive access. The reference DB IS read (read-only) for
the resolver-universe enumeration test, mirroring the pattern already used
by ``scopes.py`` itself; that test is skipped if the DB is absent.

``TestMutationG7`` is the guard's mutation-leg suite (standing rule: every
sentinel/guard is mutation-validated, ``tests/unit/sentinels/
test_superstructure.py:5-7``) — each leg feeds ``validate_crosswalk`` a
deliberately-violating fixture and asserts it reds (raises
``CrosswalkValidationError``), per the guard-register's "feed a violating
fixture" mode (``docs/superpowers/plans/2026-08-17-334-incidence-artifact.md``
§3 guard table, G7 row).
"""

from __future__ import annotations

import csv
import sys
from pathlib import Path

import pytest

_REPO_ROOT = Path(__file__).resolve().parents[3]
_TOOLS_DIR = _REPO_ROOT / "tools"
sys.path.insert(0, str(_TOOLS_DIR))

import make_fips_vintage_crosswalk as xwalk_tool  # type: ignore[import-not-found]  # noqa: E402

pytestmark = [pytest.mark.unit]

CrosswalkRow = xwalk_tool.CrosswalkRow


def _row(
    fips_engine: str,
    fips_acs2019: str = "",
    relation: str = "RENAME_RECOVERABLE",
    recoverable: bool = True,
) -> CrosswalkRow:
    return CrosswalkRow(
        fips_engine=fips_engine,
        fips_acs2019=fips_acs2019,
        relation=relation,
        vintage_note="fixture row",
        recoverable=recoverable,
    )


class TestMutationG7:
    """Each leg proves ``validate_crosswalk`` actually fires on the
    violation it claims to catch."""

    def test_the_real_crosswalk_validates_clean(self) -> None:
        """Baseline: the checked-in 17-row table is G7-clean (a guard that
        never passes anything is not a guard)."""
        xwalk_tool.validate_crosswalk(xwalk_tool.CROSSWALK_ROWS)

    def test_injectivity_leg_reds_on_shared_recoverable_target(self) -> None:
        """Two DIFFERENT recoverable=true rows claiming the SAME
        fips_acs2019 target must red — that's exactly the double-counting
        shape G7 exists to prevent (§ module docstring: two engine counties
        both reading the same source into a summed national total)."""
        rows = (
            _row("51515", "51019", recoverable=True),
            _row("51013", "51019", recoverable=True),  # same target, different engine fips
        )
        with pytest.raises(xwalk_tool.CrosswalkValidationError, match="injectivity"):
            xwalk_tool.validate_crosswalk(rows)

    def test_injectivity_leg_permits_shared_target_when_not_recoverable(self) -> None:
        """The real 02063/02066 -> 02261 SPLIT_UNRESOLVED pair shares a
        target but both are recoverable=false (never consumed as a data
        source), so it must NOT trip the guard — proves the leg is scoped
        correctly, not merely disabled."""
        rows = (
            _row("02063", "02261", relation="SPLIT_UNRESOLVED", recoverable=False),
            _row("02066", "02261", relation="SPLIT_UNRESOLVED", recoverable=False),
        )
        xwalk_tool.validate_crosswalk(rows)

    def test_declared_hole_is_never_a_target_leg(self) -> None:
        """Adding a second crosswalk row whose fips_acs2019 targets a
        DECLARED_HOLE county's own fips_engine (46102, Pine Ridge — zero
        rows at any vintage) must red: a hole has nothing to point to."""
        rows = (
            *xwalk_tool.CROSSWALK_ROWS,
            _row("99999", "46102", relation="RENAME_RECOVERABLE", recoverable=True),
        )
        with pytest.raises(xwalk_tool.CrosswalkValidationError, match="DECLARED_HOLE"):
            xwalk_tool.validate_crosswalk(rows)

    def test_partial_function_leg_reds_on_duplicate_fips_engine(self) -> None:
        """Two rows for the SAME fips_engine (conflicting or not) makes A1
        an ill-defined relation, not a partial function — must red."""
        rows = (
            _row("46102", "", relation="DECLARED_HOLE", recoverable=False),
            _row("46102", "46113", relation="RENAME_RECOVERABLE", recoverable=True),
        )
        with pytest.raises(xwalk_tool.CrosswalkValidationError, match="partial-function"):
            xwalk_tool.validate_crosswalk(rows)


class TestRowCountAndClassification:
    def test_seventeen_rows(self) -> None:
        assert len(xwalk_tool.CROSSWALK_ROWS) == 17

    def test_fips_engine_values_are_unique(self) -> None:
        fips = [row.fips_engine for row in xwalk_tool.CROSSWALK_ROWS]
        assert len(fips) == len(set(fips))

    def test_46102_present_as_declared_hole(self) -> None:
        by_fips = {row.fips_engine: row for row in xwalk_tool.CROSSWALK_ROWS}
        assert "46102" in by_fips
        pine_ridge = by_fips["46102"]
        assert pine_ridge.relation == xwalk_tool.RELATION_DECLARED_HOLE
        assert pine_ridge.recoverable is False
        assert pine_ridge.fips_acs2019 == ""

    def test_46102_is_never_recoverable_from_46113(self) -> None:
        """G5's forthcoming Pine-Ridge-imputation-forbidden leg (T3b) starts
        from A1 never offering 46113 as 46102's target."""
        by_fips = {row.fips_engine: row for row in xwalk_tool.CROSSWALK_ROWS}
        assert by_fips["46102"].fips_acs2019 != "46113"

    def test_resolver_only_delta_rows_present(self) -> None:
        by_fips = {row.fips_engine: row for row in xwalk_tool.CROSSWALK_ROWS}
        for fips in ("02261", "02270", "46113"):
            assert fips in by_fips, f"resolver-only delta row missing: {fips}"
            assert by_fips[fips].relation == xwalk_tool.RELATION_RESOLVER_ONLY

    def test_recoverable_rows_have_a_populated_target(self) -> None:
        for row in xwalk_tool.CROSSWALK_ROWS:
            if row.recoverable:
                assert row.fips_acs2019 != "", f"{row.fips_engine} recoverable but no target"


class TestUniverseEnumeration:
    def test_engine_universe_size(self) -> None:
        universe = xwalk_tool._enumerate_engine_universe(xwalk_tool.ENGINE_TERRITORIES_JSON)
        assert len(universe) == 3153

    def test_resolver_universe_size_and_delta(self) -> None:
        sqlite_path = xwalk_tool.DEFAULT_SQLITE_PATH
        if not (_REPO_ROOT / sqlite_path).exists() and not sqlite_path.exists():
            pytest.skip("reference DB not present in this environment")
        engine_universe = xwalk_tool._enumerate_engine_universe(xwalk_tool.ENGINE_TERRITORIES_JSON)
        resolver_universe = xwalk_tool._enumerate_resolver_universe(sqlite_path)
        assert len(resolver_universe) == 3156
        delta = resolver_universe - engine_universe
        assert delta == frozenset({"02261", "02270", "46113"})


class TestDeterministicEmission:
    def test_render_csv_header_and_sort_order(self) -> None:
        text = xwalk_tool._render_csv(xwalk_tool.CROSSWALK_ROWS)
        reader = csv.reader(text.splitlines())
        header = next(reader)
        assert tuple(header) == xwalk_tool.CROSSWALK_COLUMNS
        body_fips = [row[0] for row in reader]
        assert body_fips == sorted(body_fips)
        assert len(body_fips) == 17

    def test_lf_line_endings_only(self) -> None:
        text = xwalk_tool._render_csv(xwalk_tool.CROSSWALK_ROWS)
        assert "\r" not in text

    def test_generator_is_idempotent(self, tmp_path: Path) -> None:
        out1 = tmp_path / "a.csv"
        out2 = tmp_path / "b.csv"
        _rows1, sha1 = xwalk_tool._write_artifact(out1, xwalk_tool.CROSSWALK_ROWS)
        _rows2, sha2 = xwalk_tool._write_artifact(out2, xwalk_tool.CROSSWALK_ROWS)
        assert sha1 == sha2
        assert out1.read_bytes() == out2.read_bytes()

    def test_checked_in_artifact_matches_regeneration(self) -> None:
        committed = xwalk_tool.DEFAULT_OUT
        assert committed.is_file(), "checked-in crosswalk artifact missing"
        regenerated = xwalk_tool._render_csv(xwalk_tool.CROSSWALK_ROWS)
        assert committed.read_text(encoding="utf-8") == regenerated
