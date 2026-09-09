"""Unit tests for the national-incidence artifact generator skeleton
(#334 Phase 0, T2) — ``tools/make_national_incidence_artifact.py``.

T2's scope (plan §3 steps 1-2 only — "no measures yet"): sha-pinned source
access (``--export-source`` + provenance verification), a filtered pyarrow
read of the poverty cells, and resolution of the three ``universe_variant``
FIPS sets. No guard (G1-G8), no measure, no emission — those are T3a/T3b/T4.

Two test classes touch the real read-only reference DB
(``data/sqlite/marxist-data-3NF.sqlite``) against the small dimension tables
only (``dim_county`` 3,285 rows / ``dim_race`` 10 rows) — never
``fact_census_poverty`` (26.5M rows; T2's "never materialize the full 26.5M
rows" discipline extends to this test suite too). They ``pytest.skip`` if the
DB is absent (mirrors ``tests/unit/tools/test_fips_vintage_crosswalk.py``'s
established pattern) and additionally skip if the runtime SQLite isn't the
pinned 3.53.1 (``export_source_tables`` hard-gates on it — run this file's
export-touching tests via ``mise run nix -- mise run test:q -- <this file>``
to exercise them for real; off-pin they skip cleanly so the fast local loop
stays unblocked).

Everything else (provenance verification, the filtered cell read, universe
resolution, the A1 crosswalk resolver) runs against small synthetic fixtures
built with pyarrow directly — no DB, no pin, no size concern.
"""

from __future__ import annotations

import json
import sqlite3
import sys
from pathlib import Path

import pyarrow as pa
import pytest
import yaml
from pyarrow import parquet as pq

pytestmark = [pytest.mark.unit]

_REPO_ROOT = Path(__file__).resolve().parents[3]
_TOOLS_DIR = _REPO_ROOT / "tools"
sys.path.insert(0, str(_TOOLS_DIR))

import make_national_incidence_artifact as nia  # type: ignore[import-not-found]  # noqa: E402

REAL_SQLITE_PATH = _REPO_ROOT / "data/sqlite/marxist-data-3NF.sqlite"


def _skip_unless_real_db() -> None:
    if not REAL_SQLITE_PATH.exists():
        pytest.skip("reference DB not present in this environment")


def _skip_unless_pinned_sqlite() -> None:
    if sqlite3.sqlite_version != nia.PINNED_SQLITE_VERSION:
        pytest.skip(
            f"runtime sqlite3 {sqlite3.sqlite_version} != pinned "
            f"{nia.PINNED_SQLITE_VERSION} -- run via `mise exec -- uv run --frozen python -m pytest ...`"
        )


def _write_parquet(path: Path, table: pa.Table) -> str:
    path.parent.mkdir(parents=True, exist_ok=True)
    pq.write_table(table, path)
    return nia._sha256_file(path)


def _write_manifest(path: Path, entries: dict[str, str]) -> None:
    manifest = {"artifacts": [{"name": name, "sha256": sha} for name, sha in entries.items()]}
    path.write_text(yaml.safe_dump(manifest), encoding="utf-8")


class TestSourceProvenance:
    """Step 2: hard-fail on provenance drift. Pure synthetic fixtures —
    no DB, no pin dependency, always runs."""

    def test_verify_source_provenance_passes_when_shas_match(self, tmp_path: Path) -> None:
        dist_dir = tmp_path / "dist"
        table = pa.table({"race_id": [1, 2, 3], "race_code": ["T", "A", "B"]})
        sha = _write_parquet(dist_dir / "dim_race.parquet", table)
        manifest_path = tmp_path / "data-artifacts.yaml"
        _write_manifest(manifest_path, {"dim_race": sha})

        verified = nia.verify_source_provenance(
            dist_dir, manifest_path=manifest_path, tables=("dim_race",)
        )

        assert verified == {"dim_race": dist_dir / "dim_race.parquet"}

    def test_verify_source_provenance_raises_on_sha_mismatch(self, tmp_path: Path) -> None:
        dist_dir = tmp_path / "dist"
        table = pa.table({"race_id": [1, 2, 3], "race_code": ["T", "A", "B"]})
        sha = _write_parquet(dist_dir / "dim_race.parquet", table)
        manifest_path = tmp_path / "data-artifacts.yaml"
        _write_manifest(manifest_path, {"dim_race": sha})

        # Deliberately corrupt the fixture parquet AFTER pinning its sha —
        # the exact "corrupted fixture parquet" scenario the plan's T2
        # Verify line names.
        with (dist_dir / "dim_race.parquet").open("ab") as handle:
            handle.write(b"\x00corruption")

        with pytest.raises(nia.SourceProvenanceError, match="dim_race"):
            nia.verify_source_provenance(
                dist_dir, manifest_path=manifest_path, tables=("dim_race",)
            )

    def test_verify_source_provenance_raises_on_missing_source(self, tmp_path: Path) -> None:
        dist_dir = tmp_path / "dist"
        dist_dir.mkdir()
        manifest_path = tmp_path / "data-artifacts.yaml"
        _write_manifest(manifest_path, {"dim_race": "deadbeef" * 8})

        with pytest.raises(nia.SourceProvenanceError, match="dim_race"):
            nia.verify_source_provenance(
                dist_dir, manifest_path=manifest_path, tables=("dim_race",)
            )

    def test_verify_source_provenance_raises_when_manifest_entry_missing(
        self, tmp_path: Path
    ) -> None:
        dist_dir = tmp_path / "dist"
        table = pa.table({"race_id": [1]})
        _write_parquet(dist_dir / "dim_race.parquet", table)
        manifest_path = tmp_path / "data-artifacts.yaml"
        _write_manifest(manifest_path, {})  # no dim_race entry at all

        with pytest.raises(nia.ArtifactGenerationError, match="dim_race"):
            nia.verify_source_provenance(
                dist_dir, manifest_path=manifest_path, tables=("dim_race",)
            )

    def test_default_tables_are_the_four_plan_sources(self) -> None:
        assert nia.SOURCE_TABLES == (
            "fact_census_poverty",
            "dim_race",
            "dim_county",
            "dim_poverty_category",
        )

    def test_default_pins_are_loaded_from_the_real_manifest(self) -> None:
        """The pins aren't hand-typed constants in this module (drift risk,
        `docs/how-to/reference-data-pipeline.rst:63-65`'s "never hand-type a
        sha256" discipline applied to READING already-registered pins, not
        just writing new ones) — they're read straight out of the checked-in
        `data-artifacts.yaml`.

        Structural assertions only (no hand-typed sha256 literals here,
        which would themselves violate the "never hand-type a sha256"
        discipline this test exists to enforce, and would go red with no
        self-evident link to a legitimate manifest regeneration). The
        real byte-identity guarantee — that these pins actually match what
        `export_table_parquet` produces — is covered by
        `TestExportSource::test_export_source_tables_matches_registered_pins_for_small_dims`;
        this test only covers `_load_manifest_pins`'s own contract: it
        returns exactly the requested table names as keys, each mapped to
        a well-formed sha256 hex digest."""
        pins = nia._load_manifest_pins(nia.SOURCE_TABLES, nia.MANIFEST_PATH)

        assert set(pins) == set(nia.SOURCE_TABLES)
        for table, sha in pins.items():
            assert len(sha) == 64, f"{table}: sha256 should be 64 hex chars, got {len(sha)}"
            assert sha == sha.lower(), f"{table}: sha256 should be lowercase"
            assert all(c in "0123456789abcdef" for c in sha), f"{table}: not valid hex: {sha}"


class TestExportSource:
    """Step 1: --export-source mode. Touches the REAL reference DB, but only
    the small dimension tables (dim_race 10 rows, dim_county 3,285 rows) —
    never fact_census_poverty (26.5M rows)."""

    def test_export_source_tables_matches_registered_pins_for_small_dims(
        self, tmp_path: Path
    ) -> None:
        _skip_unless_real_db()
        _skip_unless_pinned_sqlite()

        out_paths = nia.export_source_tables(
            REAL_SQLITE_PATH,
            dist_dir=tmp_path,
            tables=("dim_race", "dim_poverty_category", "dim_county"),
        )

        registered = nia._load_manifest_pins(
            ("dim_race", "dim_poverty_category", "dim_county"), nia.MANIFEST_PATH
        )
        for table, path in out_paths.items():
            assert nia._sha256_file(path) == registered[table], (
                f"{table}: export_source_tables produced bytes that don't match "
                "the registered data-artifacts.yaml pin"
            )

    def test_export_source_tables_refuses_off_pin_sqlite(
        self, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        monkeypatch.setattr(nia.sqlite3, "sqlite_version", "9.9.9")

        with pytest.raises(nia.UnpinnedToolchainError, match="9.9.9"):
            nia.export_source_tables(REAL_SQLITE_PATH, dist_dir=tmp_path)

    def test_export_source_tables_raises_on_missing_db(self, tmp_path: Path) -> None:
        _skip_unless_pinned_sqlite()
        missing = tmp_path / "does-not-exist.sqlite"

        with pytest.raises(nia.ArtifactGenerationError, match="not found"):
            nia.export_source_tables(missing, dist_dir=tmp_path)


class TestFilteredRead:
    """Step 3: filtered pyarrow read. Synthetic fixtures only — proves the
    predicate (time_id=23, category_id in {1,2}, race_id in {1..10} — T4
    widened this from the plan's literal {1,2,3,4,9,10}; see
    ``nia.RACE_IDS``'s docstring for the controller ruling) actually
    excludes non-matching rows, and that fips resolution via dim_county
    works."""

    def _fixture_county_parquet(self, path: Path) -> Path:
        table = pa.table(
            {
                "county_id": pa.array([1, 2, 3], type=pa.int64()),
                "fips": ["26001", "26003", "51019"],
            }
        )
        pq.write_table(table, path)
        return path

    def _fixture_poverty_parquet(self, path: Path) -> Path:
        # A mix of matching and non-matching rows across every filtered
        # dimension (time_id, category_id, race_id) plus one row on an
        # untracked race_id (11 — outside the widened {1..10} range; T4
        # widened RACE_IDS to all ten, so 5 is now IN-scope) to prove the
        # race_id allow-list still excludes genuinely out-of-scope codes.
        table = pa.table(
            {
                "county_id": pa.array([1, 1, 1, 2, 2, 3, 3], type=pa.int64()),
                "category_id": pa.array([1, 2, 1, 1, 1, 1, 1], type=pa.int64()),
                "race_id": pa.array([1, 1, 1, 1, 1, 1, 11], type=pa.int64()),
                "time_id": pa.array([23, 23, 22, 23, 23, 23, 23], type=pa.int64()),
                "person_count": pa.array([100, 20, 999, 200, 200, 300, 999], type=pa.int64()),
            }
        )
        pq.write_table(table, path)
        return path

    def test_read_filtered_poverty_cells_applies_the_pinned_predicate(self, tmp_path: Path) -> None:
        county_parquet = self._fixture_county_parquet(tmp_path / "dim_county.parquet")
        poverty_parquet = self._fixture_poverty_parquet(tmp_path / "fact_census_poverty.parquet")

        cells = nia.read_filtered_poverty_cells(poverty_parquet, county_parquet)

        # Row 3 (time_id=22, person_count=999) excluded; row 7 (race_id=11,
        # person_count=999) excluded. The remaining five rows (two on
        # county 1 — category 1 and 2 — two duplicate rows on county 2,
        # both retained since this function does not aggregate, and one
        # on county 3) all survive.
        fips_seen = sorted(cell.fips for cell in cells)
        assert fips_seen == ["26001", "26001", "26003", "26003", "51019"]
        assert all(cell.person_count != 999 for cell in cells)

    def test_read_filtered_poverty_cells_raises_on_unjoinable_county_id(
        self, tmp_path: Path
    ) -> None:
        county_parquet = self._fixture_county_parquet(tmp_path / "dim_county.parquet")
        poverty_parquet = tmp_path / "fact_census_poverty_orphan.parquet"
        table = pa.table(
            {
                "county_id": pa.array([999], type=pa.int64()),
                "category_id": pa.array([1], type=pa.int64()),
                "race_id": pa.array([1], type=pa.int64()),
                "time_id": pa.array([23], type=pa.int64()),
                "person_count": pa.array([1], type=pa.int64()),
            }
        )
        pq.write_table(table, poverty_parquet)

        with pytest.raises(nia.ArtifactGenerationError, match="999"):
            nia.read_filtered_poverty_cells(poverty_parquet, county_parquet)


class TestCrosswalkLoadingAndResolution:
    """A1 (T1's checked-in CSV) is the authority — read directly, never
    re-derived from tools/make_fips_vintage_crosswalk.py's in-module
    constant."""

    def test_load_crosswalk_reads_the_checked_in_csv(self) -> None:
        rows = nia.load_crosswalk()

        assert len(rows) == 17
        by_engine = {row.fips_engine: row for row in rows}
        assert by_engine["46102"].relation == "DECLARED_HOLE"
        assert by_engine["46102"].recoverable is False
        assert by_engine["51515"].relation == "RENAME_RECOVERABLE"
        assert by_engine["51515"].fips_acs2019 == "51019"
        assert by_engine["51515"].recoverable is True

    def test_resolve_query_fips_substitutes_recoverable_rows(self) -> None:
        rows = nia.load_crosswalk()
        by_engine = {row.fips_engine: row for row in rows}

        assert nia.resolve_query_fips("51515", by_engine) == "51019"

    def test_resolve_query_fips_never_substitutes_declared_hole(self) -> None:
        rows = nia.load_crosswalk()
        by_engine = {row.fips_engine: row for row in rows}

        assert nia.resolve_query_fips("46102", by_engine) == "46102"

    def test_resolve_query_fips_never_substitutes_non_recoverable_rows(self) -> None:
        rows = nia.load_crosswalk()
        by_engine = {row.fips_engine: row for row in rows}

        # SPLIT_UNRESOLVED: 02063 discloses 02261 informationally but is not
        # recoverable=true, so it must NOT be substituted (T1's report:
        # "T4's derivation must not treat 02063, 02066, or 02158 as
        # recovered via crosswalk").
        assert by_engine["02063"].fips_acs2019 == "02261"
        assert by_engine["02063"].recoverable is False
        assert nia.resolve_query_fips("02063", by_engine) == "02063"

    def test_resolve_query_fips_passes_through_counties_with_no_crosswalk_row(self) -> None:
        rows = nia.load_crosswalk()
        by_engine = {row.fips_engine: row for row in rows}

        assert nia.resolve_query_fips("26001", by_engine) == "26001"


class TestUniverseResolution:
    """Step 4. The artifact/scopes universes are static-registry-derived
    (fast, no size concern); scopes additionally touches the real DB
    (skip-guarded, mirrors T1). Unrestricted uses a synthetic poverty
    fixture — the mechanism is what's under test, not the real 3,218
    figure (verified manually, see task-2-report.md)."""

    def test_artifact_universe_matches_the_engine_territory_file(self) -> None:
        payload = json.loads(nia.ENGINE_TERRITORIES_JSON.read_text(encoding="utf-8"))
        expected = frozenset(c["fips"] for c in payload["counties"])

        assert nia._artifact_universe() == expected
        assert len(expected) == 3153

    def test_scopes_universe_matches_the_resolver(self) -> None:
        _skip_unless_real_db()

        scopes_fips = nia._scopes_universe(REAL_SQLITE_PATH)

        # T1's verified finding (task-1-report.md): 3,156 counties, delta
        # {02261, 02270, 46113} against the engine's 3,153.
        assert len(scopes_fips) == 3156
        artifact_fips = nia._artifact_universe()
        assert sorted(scopes_fips - artifact_fips) == ["02261", "02270", "46113"]

    def test_unrestricted_universe_is_the_distinct_present_fips(self, tmp_path: Path) -> None:
        county_parquet = tmp_path / "dim_county.parquet"
        pq.write_table(
            pa.table(
                {
                    "county_id": pa.array([1, 2, 3], type=pa.int64()),
                    "fips": ["26001", "26003", "51019"],
                }
            ),
            county_parquet,
        )
        poverty_parquet = tmp_path / "fact_census_poverty.parquet"
        pq.write_table(
            pa.table(
                {
                    "county_id": pa.array([1, 2, 2, 3], type=pa.int64()),
                    "category_id": pa.array([1, 1, 2, 1], type=pa.int64()),
                    "race_id": pa.array([1, 1, 1, 1], type=pa.int64()),
                    # county 3's only row is time_id=22 — must NOT count as
                    # "present" in the time_id=23 unrestricted universe.
                    "time_id": pa.array([23, 23, 23, 22], type=pa.int64()),
                    "person_count": pa.array([1, 1, 1, 1], type=pa.int64()),
                }
            ),
            poverty_parquet,
        )

        result = nia._unrestricted_universe(poverty_parquet, county_parquet)

        assert result == frozenset({"26001", "26003"})

    def test_resolve_universe_variants_returns_three_named_variants(self, tmp_path: Path) -> None:
        _skip_unless_real_db()
        county_parquet = tmp_path / "dim_county.parquet"
        pq.write_table(
            pa.table({"county_id": pa.array([1], type=pa.int64()), "fips": ["26001"]}),
            county_parquet,
        )
        poverty_parquet = tmp_path / "fact_census_poverty.parquet"
        pq.write_table(
            pa.table(
                {
                    "county_id": pa.array([1], type=pa.int64()),
                    "time_id": pa.array([23], type=pa.int64()),
                }
            ),
            poverty_parquet,
        )

        variants = nia.resolve_universe_variants(
            sqlite_path=REAL_SQLITE_PATH,
            poverty_parquet=poverty_parquet,
            county_parquet=county_parquet,
        )

        assert [v.name for v in variants] == ["artifact", "scopes", "unrestricted"]
        assert len(variants[0].fips) == 3153
        assert len(variants[1].fips) == 3156
        assert variants[2].fips == frozenset({"26001"})


class TestNoFromSqliteDerivationPath:
    """Carried-forward requirement: there is no --from-sqlite derivation
    path. The main() CLI surface must not offer one."""

    def test_cli_has_no_from_sqlite_flag(self) -> None:
        option_strings = {
            option for action in nia.build_arg_parser()._actions for option in action.option_strings
        }

        assert "--from-sqlite" not in option_strings
        assert "--export-source" in option_strings
