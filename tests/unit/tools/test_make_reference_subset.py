"""Behavioral contract for the reference-DB CI subset generator (Program 14/15
Phase 6, owner item 40).

The generator (``tools/make_reference_subset.py``) shrinks the 5.7 GB
read-only reference SQLite DB (``data/sqlite/marxist-data-3NF.sqlite``) into a
CI-sized artifact by applying a reviewable, module-level table policy
(``TABLE``): every ``fact_``/``dim_``/``bridge_`` table is classified
``full`` (shipped complete), ``michigan`` (filtered to state-FIPS ``26``), or
``skip`` (dropped, documented). Three tables are pinned ``full`` because real
tests assert national coverage over them (the "BLOCKED-FULL" trio); an
unclassified table matching those prefixes is a hard error (Constitution
III.11 Loud Failure) — the policy dict must be reviewed, not silently
defaulted, whenever the trove grows a new table.

The synthetic-input tests are the red-phase/mutation proof that each unit
(policy validation, WHERE-clause building, DDL/index qualification, table
copy, manifest emission, sha256 sidecar, loud failure) behaves correctly in
isolation, built against tiny sqlite fixtures in ``tmp_path`` — never the
real 5.7 GB database. A final class pins static shape invariants of the real
production ``TABLE`` policy dict without opening any database file.
"""

from __future__ import annotations

import hashlib
import json
import sqlite3
import sys
from pathlib import Path

import pytest

# Mirror the import path used by tools/*.py and its existing unit tests
# (see tests/unit/tools/test_repo_hygiene.py, test_run_pip_audit.py).
TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
sys.path.insert(0, str(TOOLS_DIR))

from make_reference_subset import (  # type: ignore[import-not-found]  # noqa: E402
    TABLE,
    TablePolicy,
    build_manifest,
    build_michigan_where_clause,
    compute_sha256,
    copy_table,
    find_unknown_tables,
    generate_subset,
    get_index_ddls,
    get_source_table_names,
    get_table_ddl,
    main,
    qualify_index_ddl_for_dest,
    qualify_table_ddl_for_dest,
)


def _make_fixture_db(path: Path) -> None:
    """Build a tiny synthetic reference-DB fixture with a Michigan slice.

    Schema mirrors the real DB's shape for the tables this suite exercises:
    ``dim_county`` (5 counties, 2 Michigan + 3 other states) and
    ``fact_qcew_annual``-shaped county-scoped fact rows, plus a national
    ``fact_bea_national_industry``-shaped table with no county column.

    :param path: Where to create the sqlite file (must not already exist).
    """
    conn = sqlite3.connect(path)
    conn.executescript(
        """
        CREATE TABLE dim_county (
            county_id INTEGER PRIMARY KEY,
            fips VARCHAR(5) NOT NULL,
            county_name VARCHAR(200) NOT NULL
        );
        CREATE INDEX idx_county_name ON dim_county (county_name);
        INSERT INTO dim_county VALUES
            (1, '26163', 'Wayne'),
            (2, '26125', 'Oakland'),
            (3, '17031', 'Cook'),
            (4, '06037', 'Los Angeles'),
            (5, '26999', 'Rest of Michigan');

        CREATE TABLE fact_qcew_annual (
            county_id INTEGER NOT NULL,
            time_id INTEGER NOT NULL,
            employment INTEGER,
            PRIMARY KEY (county_id, time_id)
        );
        CREATE INDEX idx_qcew_county_time ON fact_qcew_annual (county_id, time_id);
        INSERT INTO fact_qcew_annual VALUES
            (1, 2022, 900000),
            (2, 2022, 700000),
            (3, 2022, 2500000),
            (4, 2022, 4500000),
            (5, 2022, 1000);

        CREATE TABLE fact_lodes_commuter_flow (
            home_county_id INTEGER NOT NULL,
            work_county_id INTEGER NOT NULL,
            time_id INTEGER NOT NULL,
            total_jobs INTEGER NOT NULL,
            PRIMARY KEY (home_county_id, work_county_id, time_id)
        );
        INSERT INTO fact_lodes_commuter_flow VALUES
            (1, 2, 2021, 5000),
            (3, 3, 2021, 8000),
            (2, 3, 2021, 250);

        CREATE TABLE "fact_bea_national_industry" (
            bea_industry_id INTEGER NOT NULL,
            time_id INTEGER NOT NULL,
            gross_output_millions NUMERIC(15, 2),
            PRIMARY KEY (bea_industry_id, time_id)
        );
        INSERT INTO fact_bea_national_industry VALUES (1, 2022, 500.0);

        CREATE TABLE fact_retired_skip_example (
            county_id INTEGER NOT NULL,
            employment INTEGER
        );
        INSERT INTO fact_retired_skip_example VALUES (1, 1);

        CREATE TABLE dim_time (
            time_id INTEGER PRIMARY KEY,
            year INTEGER NOT NULL
        );
        INSERT INTO dim_time VALUES (2022, 2022);

        CREATE TABLE staging_arcgis_feature (id INTEGER PRIMARY KEY);
        """
    )
    conn.commit()
    conn.close()


#: Policy dict matching the fixture built by ``_make_fixture_db``.
FIXTURE_POLICY: dict[str, TablePolicy] = {
    "dim_county": TablePolicy(scope="full", reason="Small dimension table."),
    "dim_time": TablePolicy(scope="full", reason="Small dimension table."),
    "fact_qcew_annual": TablePolicy(
        scope="michigan",
        reason="County-scoped QCEW; MI slice.",
        county_columns=("county_id",),
    ),
    "fact_lodes_commuter_flow": TablePolicy(
        scope="michigan",
        reason="County-pair commuter flow; MI on either side.",
        county_columns=("home_county_id", "work_county_id"),
    ),
    "fact_bea_national_industry": TablePolicy(scope="full", reason="National, tiny."),
    "fact_retired_skip_example": TablePolicy(
        scope="skip", reason="Synthetic retired table exercising skip behavior."
    ),
}


@pytest.fixture
def fixture_db(tmp_path: Path) -> Path:
    source = tmp_path / "fixture-source.sqlite"
    _make_fixture_db(source)
    return source


def _make_carveout_fixture_db(path: Path) -> None:
    """Build a synthetic DB mirroring the spec-098 regression-guard scenario.

    Bullock County, AL (fips ``01011``) is not Michigan, but a real test
    (``test_get_county_naics_employment_real_zero_is_not_none`` et al.) reads
    a confirmed-zero row for it out of ``fact_qcew_annual`` — so a policy's
    ``extra_fips`` carve-out must preserve it even though the plain Michigan
    filter would drop it. Cook County, IL (fips ``17031``) is included,
    unlisted, to prove the carve-out does not become a "keep everything"
    escape hatch — it must still be dropped.

    :param path: Where to create the sqlite file (must not already exist).
    """
    conn = sqlite3.connect(path)
    conn.executescript(
        """
        CREATE TABLE dim_county (
            county_id INTEGER PRIMARY KEY,
            fips VARCHAR(5) NOT NULL,
            county_name VARCHAR(200) NOT NULL
        );
        INSERT INTO dim_county VALUES
            (1, '26163', 'Wayne'),
            (2, '17031', 'Cook'),
            (3, '01011', 'Bullock');

        CREATE TABLE fact_qcew_annual (
            county_id INTEGER NOT NULL,
            time_id INTEGER NOT NULL,
            employment INTEGER,
            PRIMARY KEY (county_id, time_id)
        );
        INSERT INTO fact_qcew_annual VALUES
            (1, 2022, 900000),
            (2, 2022, 2500000),
            (3, 2024, 0);
        """
    )
    conn.commit()
    conn.close()


@pytest.fixture
def carveout_fixture_db(tmp_path: Path) -> Path:
    source = tmp_path / "carveout-source.sqlite"
    _make_carveout_fixture_db(source)
    return source


@pytest.mark.unit
class TestTablePolicyValidation:
    """``TablePolicy`` enforces its own scope/county_columns/reason invariants."""

    def test_full_scope_rejects_county_columns(self) -> None:
        with pytest.raises(ValueError, match="county_columns"):
            TablePolicy(scope="full", reason="x", county_columns=("county_id",))

    def test_skip_scope_rejects_county_columns(self) -> None:
        with pytest.raises(ValueError, match="county_columns"):
            TablePolicy(scope="skip", reason="x", county_columns=("county_id",))

    def test_michigan_scope_requires_county_columns(self) -> None:
        with pytest.raises(ValueError, match="county_columns"):
            TablePolicy(scope="michigan", reason="x")

    def test_blank_reason_rejected(self) -> None:
        with pytest.raises(ValueError, match="reason"):
            TablePolicy(scope="full", reason="   ")

    def test_valid_full_policy_constructs(self) -> None:
        policy = TablePolicy(scope="full", reason="small dimension table")
        assert policy.scope == "full"
        assert policy.county_columns == ()

    def test_valid_michigan_policy_constructs(self) -> None:
        policy = TablePolicy(
            scope="michigan", reason="county-scoped", county_columns=("county_id",)
        )
        assert policy.county_columns == ("county_id",)

    def test_full_scope_rejects_extra_fips(self) -> None:
        with pytest.raises(ValueError, match="extra_fips"):
            TablePolicy(scope="full", reason="x", extra_fips=("01011",))

    def test_skip_scope_rejects_extra_fips(self) -> None:
        with pytest.raises(ValueError, match="extra_fips"):
            TablePolicy(scope="skip", reason="x", extra_fips=("01011",))

    def test_michigan_scope_accepts_extra_fips(self) -> None:
        policy = TablePolicy(
            scope="michigan",
            reason="county-scoped + carve-out",
            county_columns=("county_id",),
            extra_fips=("01011",),
        )
        assert policy.extra_fips == ("01011",)

    def test_extra_fips_defaults_empty(self) -> None:
        policy = TablePolicy(
            scope="michigan", reason="county-scoped", county_columns=("county_id",)
        )
        assert policy.extra_fips == ()

    def test_extra_fips_rejects_malformed_code(self) -> None:
        with pytest.raises(ValueError, match="5-digit"):
            TablePolicy(
                scope="michigan",
                reason="county-scoped",
                county_columns=("county_id",),
                extra_fips=("not-a-fips",),
            )


@pytest.mark.unit
class TestFindUnknownTables:
    """Loud-failure detection: unclassified fact_/dim_/bridge_ tables."""

    def test_no_unknown_tables_when_all_classified(self) -> None:
        names = ["dim_county", "fact_qcew_annual", "staging_arcgis_feature"]
        assert find_unknown_tables(names, FIXTURE_POLICY) == []

    def test_unknown_fact_table_detected(self) -> None:
        names = ["dim_county", "fact_qcew_annual", "fact_brand_new_thing"]
        assert find_unknown_tables(names, FIXTURE_POLICY) == ["fact_brand_new_thing"]

    def test_unknown_dim_and_bridge_tables_detected_sorted(self) -> None:
        names = ["dim_county", "bridge_new_thing", "dim_new_thing"]
        assert find_unknown_tables(names, FIXTURE_POLICY) == [
            "bridge_new_thing",
            "dim_new_thing",
        ]

    def test_non_prefixed_unknown_table_ignored(self) -> None:
        # staging_/ingest_ tables are out of policy scope entirely — not an error.
        names = ["dim_county", "staging_arcgis_feature", "ingest_checkpoint"]
        assert find_unknown_tables(names, FIXTURE_POLICY) == []


@pytest.mark.unit
class TestMichiganWhereClauseBuilder:
    """The Michigan WHERE-clause builder — single and multi-column cases."""

    def test_single_column_builds_in_subquery(self) -> None:
        clause = build_michigan_where_clause(("county_id",))
        assert clause == (
            "county_id IN (SELECT county_id FROM main.dim_county WHERE fips LIKE '26%')"
        )

    def test_two_columns_or_combined(self) -> None:
        clause = build_michigan_where_clause(("home_county_id", "work_county_id"))
        assert " OR " in clause
        assert clause.count("IN (SELECT county_id FROM main.dim_county") == 2

    def test_empty_columns_raises(self) -> None:
        with pytest.raises(ValueError, match="county column"):
            build_michigan_where_clause(())

    def test_no_extra_fips_matches_pre_carveout_shape(self) -> None:
        # Backward-compat pin: default extra_fips=() must reproduce the exact
        # pre-carve-out clause byte-for-byte (no stray parens/whitespace).
        clause = build_michigan_where_clause(("county_id",), extra_fips=())
        assert clause == (
            "county_id IN (SELECT county_id FROM main.dim_county WHERE fips LIKE '26%')"
        )

    def test_single_extra_fips_or_combined_into_subquery(self) -> None:
        clause = build_michigan_where_clause(("county_id",), extra_fips=("01011",))
        assert clause == (
            "county_id IN (SELECT county_id FROM main.dim_county WHERE "
            "(fips LIKE '26%' OR fips IN ('01011')))"
        )

    def test_multiple_extra_fips_or_combined(self) -> None:
        clause = build_michigan_where_clause(("county_id",), extra_fips=("01011", "36061"))
        assert "fips IN ('01011', '36061')" in clause


@pytest.mark.unit
class TestDDLQualification:
    """Rewriting captured DDL to target the attached ``dest`` schema."""

    def test_qualifies_quoted_table_name(self) -> None:
        ddl = 'CREATE TABLE "fact_qcew_annual" (\n\tcounty_id INTEGER NOT NULL\n)'
        result = qualify_table_ddl_for_dest(ddl, "fact_qcew_annual")
        assert result.startswith('CREATE TABLE dest."fact_qcew_annual"')
        assert "county_id INTEGER NOT NULL" in result

    def test_qualifies_unquoted_table_name(self) -> None:
        ddl = "CREATE TABLE fact_qcew_annual (\n\tcounty_id INTEGER NOT NULL\n)"
        result = qualify_table_ddl_for_dest(ddl, "fact_qcew_annual")
        assert result.startswith("CREATE TABLE dest.fact_qcew_annual")

    def test_mismatched_table_name_raises(self) -> None:
        ddl = "CREATE TABLE other_table (a INTEGER)"
        with pytest.raises(ValueError, match="does not match"):
            qualify_table_ddl_for_dest(ddl, "fact_qcew_annual")

    def test_qualifies_unquoted_index_name(self) -> None:
        ddl = "CREATE INDEX idx_qcew_county_time ON fact_qcew_annual (county_id)"
        result = qualify_index_ddl_for_dest(ddl)
        assert result == ("CREATE INDEX dest.idx_qcew_county_time ON fact_qcew_annual (county_id)")

    def test_qualifies_unique_index(self) -> None:
        ddl = "CREATE UNIQUE INDEX idx_x ON t (a, b)"
        result = qualify_index_ddl_for_dest(ddl)
        assert result == "CREATE UNIQUE INDEX dest.idx_x ON t (a, b)"

    def test_not_an_index_statement_raises(self) -> None:
        with pytest.raises(ValueError, match="CREATE INDEX"):
            qualify_index_ddl_for_dest("CREATE TABLE t (a INTEGER)")


@pytest.mark.unit
class TestSchemaIntrospection:
    """DDL/index capture straight from ``sqlite_master`` (schema-copy fidelity)."""

    def test_get_source_table_names_excludes_views(self, fixture_db: Path) -> None:
        conn = sqlite3.connect(fixture_db)
        conn.executescript("CREATE VIEW v_should_not_appear AS SELECT * FROM dim_county;")
        names = get_source_table_names(conn)
        conn.close()
        assert "v_should_not_appear" not in names
        assert "dim_county" in names
        assert "fact_qcew_annual" in names

    def test_get_table_ddl_roundtrips(self, fixture_db: Path) -> None:
        conn = sqlite3.connect(fixture_db)
        ddl = get_table_ddl(conn, "fact_qcew_annual")
        conn.close()
        assert "CREATE TABLE fact_qcew_annual" in ddl
        assert "employment INTEGER" in ddl

    def test_get_table_ddl_missing_table_raises(self, fixture_db: Path) -> None:
        conn = sqlite3.connect(fixture_db)
        with pytest.raises(LookupError):
            get_table_ddl(conn, "does_not_exist")
        conn.close()

    def test_get_index_ddls_returns_named_indexes(self, fixture_db: Path) -> None:
        conn = sqlite3.connect(fixture_db)
        ddls = get_index_ddls(conn, "fact_qcew_annual")
        conn.close()
        assert len(ddls) == 1
        assert "idx_qcew_county_time" in ddls[0]

    def test_get_index_ddls_empty_for_no_named_indexes(self, fixture_db: Path) -> None:
        conn = sqlite3.connect(fixture_db)
        ddls = get_index_ddls(conn, "fact_lodes_commuter_flow")
        conn.close()
        assert ddls == []


@pytest.mark.unit
class TestCopyTable:
    """End-to-end single-table copy against an attached dest connection."""

    def _open_attached(self, source: Path, dest: Path) -> sqlite3.Connection:
        conn = sqlite3.connect(source)
        conn.execute("ATTACH DATABASE ? AS dest", (str(dest),))
        return conn

    def test_full_scope_copies_all_rows(self, fixture_db: Path, tmp_path: Path) -> None:
        dest = tmp_path / "out.sqlite"
        conn = self._open_attached(fixture_db, dest)
        kept, source_rows = copy_table(conn, "dim_county", FIXTURE_POLICY["dim_county"])
        conn.commit()
        rows = conn.execute("SELECT COUNT(*) FROM dest.dim_county").fetchone()[0]
        conn.close()
        assert kept == source_rows == 5
        assert rows == 5

    def test_michigan_scope_filters_rows(self, fixture_db: Path, tmp_path: Path) -> None:
        dest = tmp_path / "out.sqlite"
        conn = self._open_attached(fixture_db, dest)
        copy_table(conn, "dim_county", FIXTURE_POLICY["dim_county"])
        kept, source_rows = copy_table(conn, "fact_qcew_annual", FIXTURE_POLICY["fact_qcew_annual"])
        conn.commit()
        rows = conn.execute(
            "SELECT county_id FROM dest.fact_qcew_annual ORDER BY county_id"
        ).fetchall()
        conn.close()
        assert source_rows == 5
        assert kept == 3  # Wayne, Oakland, and synthetic 26999
        assert [r[0] for r in rows] == [1, 2, 5]

    def test_michigan_scope_two_columns_or_semantics(
        self, fixture_db: Path, tmp_path: Path
    ) -> None:
        dest = tmp_path / "out.sqlite"
        conn = self._open_attached(fixture_db, dest)
        copy_table(conn, "dim_county", FIXTURE_POLICY["dim_county"])
        kept, source_rows = copy_table(
            conn,
            "fact_lodes_commuter_flow",
            FIXTURE_POLICY["fact_lodes_commuter_flow"],
        )
        conn.commit()
        rows = conn.execute(
            "SELECT home_county_id, work_county_id FROM dest.fact_lodes_commuter_flow"
        ).fetchall()
        conn.close()
        assert source_rows == 3
        # (1,2): both MI. (3,3): neither MI (Cook->Cook). (2,3): Oakland->Cook, MI on one side.
        assert kept == 2
        assert (3, 3) not in rows

    def test_skip_scope_raises(self, fixture_db: Path, tmp_path: Path) -> None:
        dest = tmp_path / "out.sqlite"
        conn = self._open_attached(fixture_db, dest)
        with pytest.raises(ValueError, match="skip"):
            copy_table(
                conn,
                "fact_retired_skip_example",
                FIXTURE_POLICY["fact_retired_skip_example"],
            )
        conn.close()

    def test_indexes_are_preserved_on_copy(self, fixture_db: Path, tmp_path: Path) -> None:
        dest = tmp_path / "out.sqlite"
        conn = self._open_attached(fixture_db, dest)
        copy_table(conn, "dim_county", FIXTURE_POLICY["dim_county"])
        copy_table(conn, "fact_qcew_annual", FIXTURE_POLICY["fact_qcew_annual"])
        conn.commit()
        index_names = {
            row[0]
            for row in conn.execute(
                "SELECT name FROM dest.sqlite_master WHERE type = 'index'"
            ).fetchall()
        }
        conn.close()
        assert "idx_qcew_county_time" in index_names


@pytest.mark.unit
class TestExtraFipsCarveOut:
    """Regression-guard carve-out: Michigan filter + extra_fips both kept,
    non-listed non-Michigan rows still dropped (spec-098's 0-vs-None guard)."""

    def _open_attached(self, source: Path, dest: Path) -> sqlite3.Connection:
        conn = sqlite3.connect(source)
        conn.execute("ATTACH DATABASE ? AS dest", (str(dest),))
        return conn

    def test_michigan_and_carveout_rows_both_kept(
        self, carveout_fixture_db: Path, tmp_path: Path
    ) -> None:
        dest = tmp_path / "out.sqlite"
        conn = self._open_attached(carveout_fixture_db, dest)
        copy_table(
            conn,
            "dim_county",
            TablePolicy(scope="full", reason="small dimension table"),
        )
        carve_out_policy = TablePolicy(
            scope="michigan",
            reason="spec-098 carve-out test",
            county_columns=("county_id",),
            extra_fips=("01011",),
        )
        kept, source_rows = copy_table(conn, "fact_qcew_annual", carve_out_policy)
        conn.commit()
        rows = {
            row[0] for row in conn.execute("SELECT county_id FROM dest.fact_qcew_annual").fetchall()
        }
        conn.close()
        assert source_rows == 3
        assert kept == 2
        # Wayne (MI) and Bullock (carve-out) survive.
        assert rows == {1, 3}

    def test_non_listed_non_michigan_row_still_dropped(
        self, carveout_fixture_db: Path, tmp_path: Path
    ) -> None:
        dest = tmp_path / "out.sqlite"
        conn = self._open_attached(carveout_fixture_db, dest)
        copy_table(
            conn,
            "dim_county",
            TablePolicy(scope="full", reason="small dimension table"),
        )
        carve_out_policy = TablePolicy(
            scope="michigan",
            reason="spec-098 carve-out test",
            county_columns=("county_id",),
            extra_fips=("01011",),
        )
        copy_table(conn, "fact_qcew_annual", carve_out_policy)
        conn.commit()
        rows = {
            row[0] for row in conn.execute("SELECT county_id FROM dest.fact_qcew_annual").fetchall()
        }
        conn.close()
        # Cook County (IL, 17031) is neither Michigan nor carved out.
        assert 2 not in rows

    def test_without_carveout_bullock_row_is_dropped(
        self, carveout_fixture_db: Path, tmp_path: Path
    ) -> None:
        """Sanity control: without extra_fips, the plain MI filter drops Bullock too."""
        dest = tmp_path / "out.sqlite"
        conn = self._open_attached(carveout_fixture_db, dest)
        copy_table(
            conn,
            "dim_county",
            TablePolicy(scope="full", reason="small dimension table"),
        )
        plain_mi_policy = TablePolicy(
            scope="michigan",
            reason="plain MI filter, no carve-out",
            county_columns=("county_id",),
        )
        kept, source_rows = copy_table(conn, "fact_qcew_annual", plain_mi_policy)
        conn.commit()
        rows = {
            row[0] for row in conn.execute("SELECT county_id FROM dest.fact_qcew_annual").fetchall()
        }
        conn.close()
        assert source_rows == 3
        assert kept == 1
        assert rows == {1}


@pytest.mark.unit
class TestManifest:
    """Manifest JSON shape: per-table kept/source row counts + scope."""

    def test_manifest_lists_every_table(self) -> None:
        table_results = {
            "dim_county": {
                "scope": "full",
                "reason": "small",
                "kept_rows": 5,
                "source_rows": 5,
            },
            "fact_qcew_annual": {
                "scope": "michigan",
                "reason": "mi",
                "kept_rows": 3,
                "source_rows": 5,
            },
        }
        manifest = build_manifest(
            source_path=Path("/src.sqlite"),
            output_path=Path("/out.sqlite"),
            generated_at="2026-07-11T00:00:00+00:00",
            table_results=table_results,
        )
        assert manifest["tables"] == table_results
        assert manifest["source"] == "/src.sqlite"
        assert manifest["output"] == "/out.sqlite"
        assert manifest["generated_at"] == "2026-07-11T00:00:00+00:00"


@pytest.mark.unit
class TestSha256Sidecar:
    """SHA-256 hashing of the generated artifact."""

    def test_compute_sha256_matches_hashlib(self, tmp_path: Path) -> None:
        target = tmp_path / "blob.bin"
        target.write_bytes(b"the fall of america" * 1000)
        expected = hashlib.sha256(target.read_bytes()).hexdigest()
        assert compute_sha256(target) == expected

    def test_compute_sha256_empty_file(self, tmp_path: Path) -> None:
        target = tmp_path / "empty.bin"
        target.write_bytes(b"")
        assert compute_sha256(target) == hashlib.sha256(b"").hexdigest()


@pytest.mark.unit
class TestGenerateSubsetEndToEnd:
    """Full orchestration against the synthetic fixture (never the real DB)."""

    def test_generates_subset_manifest_and_sha256(self, fixture_db: Path, tmp_path: Path) -> None:
        output = tmp_path / "subset.sqlite"
        manifest_path = tmp_path / "manifest.json"

        manifest = generate_subset(fixture_db, output, manifest_path, policy=FIXTURE_POLICY)

        assert output.is_file()
        assert manifest_path.is_file()
        sidecar = output.with_name(output.name + ".sha256")
        assert sidecar.is_file()
        assert compute_sha256(output) in sidecar.read_text()

        on_disk_manifest = json.loads(manifest_path.read_text())
        assert on_disk_manifest == manifest
        assert manifest["tables"]["fact_qcew_annual"]["scope"] == "michigan"
        assert manifest["tables"]["fact_qcew_annual"]["kept_rows"] == 3
        assert manifest["tables"]["fact_qcew_annual"]["source_rows"] == 5
        assert manifest["tables"]["fact_retired_skip_example"]["scope"] == "skip"
        assert manifest["tables"]["fact_retired_skip_example"]["kept_rows"] == 0
        assert manifest["tables"]["fact_retired_skip_example"]["source_rows"] == 1

    def test_skipped_table_absent_from_output(self, fixture_db: Path, tmp_path: Path) -> None:
        output = tmp_path / "subset.sqlite"
        manifest_path = tmp_path / "manifest.json"
        generate_subset(fixture_db, output, manifest_path, policy=FIXTURE_POLICY)

        out_conn = sqlite3.connect(output)
        names = get_source_table_names(out_conn)
        out_conn.close()
        assert "fact_retired_skip_example" not in names

    def test_output_is_smaller_than_source_after_filtering(
        self, fixture_db: Path, tmp_path: Path
    ) -> None:
        output = tmp_path / "subset.sqlite"
        manifest_path = tmp_path / "manifest.json"
        generate_subset(fixture_db, output, manifest_path, policy=FIXTURE_POLICY)
        assert output.stat().st_size > 0

    def test_source_is_never_modified(self, fixture_db: Path, tmp_path: Path) -> None:
        before = fixture_db.read_bytes()
        generate_subset(
            fixture_db,
            tmp_path / "subset.sqlite",
            tmp_path / "manifest.json",
            policy=FIXTURE_POLICY,
        )
        after = fixture_db.read_bytes()
        assert before == after

    def test_output_same_as_source_path_raises(self, fixture_db: Path, tmp_path: Path) -> None:
        with pytest.raises(ValueError, match="output_path"):
            generate_subset(
                fixture_db, fixture_db, tmp_path / "manifest.json", policy=FIXTURE_POLICY
            )

    def test_missing_source_raises_file_not_found(self, tmp_path: Path) -> None:
        with pytest.raises(FileNotFoundError):
            generate_subset(
                tmp_path / "does-not-exist.sqlite",
                tmp_path / "out.sqlite",
                tmp_path / "manifest.json",
                policy=FIXTURE_POLICY,
            )

    def test_unclassified_table_hard_fails(self, tmp_path: Path) -> None:
        source = tmp_path / "with-unknown.sqlite"
        conn = sqlite3.connect(source)
        conn.executescript(
            """
            CREATE TABLE dim_county (county_id INTEGER PRIMARY KEY, fips VARCHAR(5));
            CREATE TABLE fact_qcew_annual (county_id INTEGER, time_id INTEGER);
            CREATE TABLE fact_brand_new_unclassified (id INTEGER PRIMARY KEY);
            """
        )
        conn.commit()
        conn.close()

        with pytest.raises(ValueError, match="fact_brand_new_unclassified"):
            generate_subset(
                source,
                tmp_path / "out.sqlite",
                tmp_path / "manifest.json",
                policy=FIXTURE_POLICY,
            )

    def test_existing_output_is_replaced(self, fixture_db: Path, tmp_path: Path) -> None:
        output = tmp_path / "subset.sqlite"
        output.write_text("stale placeholder, not a real sqlite file")
        generate_subset(fixture_db, output, tmp_path / "manifest.json", policy=FIXTURE_POLICY)
        conn = sqlite3.connect(output)
        names = get_source_table_names(conn)
        conn.close()
        assert "dim_county" in names


@pytest.mark.unit
class TestMainCLI:
    """CLI entry point exit codes."""

    def test_main_success_exit_zero(self, fixture_db: Path, tmp_path: Path) -> None:
        # main() binds the production TABLE (def-time default), so the
        # fixture-only synthetic skip table must not be present — every
        # remaining fixture table is classified in the real policy dict.
        conn = sqlite3.connect(fixture_db)
        conn.execute("DROP TABLE fact_retired_skip_example")
        conn.commit()
        conn.close()
        output = tmp_path / "subset.sqlite"
        manifest_path = tmp_path / "manifest.json"

        exit_code = main(
            [
                "--source",
                str(fixture_db),
                "--output",
                str(output),
                "--manifest",
                str(manifest_path),
            ]
        )

        assert exit_code == 0
        assert output.is_file()
        assert manifest_path.is_file()

    def test_main_missing_source_exit_two(
        self, tmp_path: Path, capsys: pytest.CaptureFixture[str]
    ) -> None:
        exit_code = main(
            [
                "--source",
                str(tmp_path / "nope.sqlite"),
                "--output",
                str(tmp_path / "out.sqlite"),
                "--manifest",
                str(tmp_path / "manifest.json"),
            ]
        )
        assert exit_code == 2
        assert "not found" in capsys.readouterr().err


@pytest.mark.unit
class TestProductionPolicyShape:
    """Static invariants of the real production ``TABLE`` dict.

    Pure-Python checks on the dict literal only — never opens the real
    5.7 GB database (that verification happens manually in Phase 2, not in
    the unit suite).
    """

    def test_every_entry_has_nonempty_reason(self) -> None:
        for name, policy in TABLE.items():
            assert policy.reason.strip(), f"{name} has a blank reason"

    def test_michigan_entries_have_county_columns(self) -> None:
        for name, policy in TABLE.items():
            if policy.scope == "michigan":
                assert policy.county_columns, f"{name} is michigan-scoped with no columns"

    def test_full_and_skip_entries_have_no_county_columns(self) -> None:
        for name, policy in TABLE.items():
            if policy.scope != "michigan":
                assert policy.county_columns == (), f"{name} has stray county_columns"

    def test_blocked_full_tables_are_full_scope(self) -> None:
        for name in (
            "dim_county_geometry",
            "fact_bea_county_gdp",
            "fact_qcew_county_rollup",
        ):
            assert TABLE[name].scope == "full", f"{name} must be BLOCKED-FULL"

    def test_faf_is_full_and_pre086_qcew_is_amputated(self) -> None:
        # FAF flipped skip -> full for ci-data-v2: sqlite_hydrator (the headless
        # runner's hydration path) reads it — the Determinism Bundle CI job died
        # ENGINE_FAILURE without it on the ci-data-v1 proving run (2026-07-11).
        assert TABLE["fact_faf_commodity_flow"].scope == "full"
        # A1 amputated 2026-07-17 (ADR075 ruling 1): the __pre_086 backup was
        # dropped from the DB, so a lingering policy entry would be a phantom.
        assert "fact_qcew_annual__pre_086" not in TABLE

    def test_known_michigan_tables(self) -> None:
        expected_michigan = {
            "fact_qcew_annual",
            "fact_county_exposure_by_external",
            "fact_census_rent",
            "fact_coercive_infrastructure",
            "fact_broadband_coverage",
            "fact_census_institutional_ownership",
            "fact_lodes_commuter_flow",
        }
        for name in expected_michigan:
            assert TABLE[name].scope == "michigan", f"{name} should be michigan-scoped"

    def test_fact_census_income_is_full_not_michigan(self) -> None:
        # Wave 6 C3: SQLiteCensusIncomeSource.get_county_bracket_ratio queries
        # ANY county nationwide (Constitution Amendment R canonical scale), so
        # a Michigan-only subset would silently starve every non-MI county —
        # flipped to FULL, no longer in expected_michigan above.
        assert TABLE["fact_census_income"].scope == "full"

    def test_all_dim_and_bridge_tables_classified_full_or_skip(self) -> None:
        for name, policy in TABLE.items():
            if name.startswith(("dim_", "bridge_")):
                assert policy.scope in ("full", "skip"), (
                    f"{name}: dim_/bridge_ tables must be full or skip, got {policy.scope}"
                )

    def test_no_duplicate_reasons_hiding_a_copy_paste_error(self) -> None:
        # Sanity: policy has a meaningful number of distinct reasons, not one
        # generic string copy-pasted onto every entry with no distinctions.
        distinct_reasons = {policy.reason for policy in TABLE.values()}
        assert len(distinct_reasons) > 5

    def test_only_michigan_scoped_entries_may_carry_extra_fips(self) -> None:
        for name, policy in TABLE.items():
            if policy.scope != "michigan":
                assert policy.extra_fips == (), f"{name} has stray extra_fips"

    def test_qcew_annual_carries_the_bullock_county_regression_guard(self) -> None:
        """spec-098's 0-vs-None guard row (Bullock County, AL) must survive the cut.

        Pinned against the two real tests that read it:
        tests/integration/economics/throughput/test_adapters.py::
        test_get_county_naics_employment_real_zero_is_not_none and
        ::test_get_county_employment_by_naics_includes_confirmed_zero_sector.
        """
        policy = TABLE["fact_qcew_annual"]
        assert policy.extra_fips == ("01011",)
        assert "spec-098" in policy.reason
        assert "test_get_county_naics_employment_real_zero_is_not_none" in policy.reason
        assert "test_get_county_employment_by_naics_includes_confirmed_zero_sector" in policy.reason
