"""Task 7: the reference-DB content round-trip verifier — canonicalization
rules, per-object verdicts, and the CLI exit-1-on-mismatch contract (parquet
pipeline plan 2026-07-19).

All fixtures are synthetic temp SQLite databases; the real reference DB is
never touched.
"""

from __future__ import annotations

import sqlite3
import sys
from pathlib import Path

import pytest

_REPO_ROOT = Path(__file__).resolve().parents[3]
TOOLS_DIR = _REPO_ROOT / "tools"
sys.path.insert(0, str(TOOLS_DIR))

import verify_reference_roundtrip  # type: ignore[import-not-found]  # noqa: E402
from build_reference_db import (  # type: ignore[import-not-found]  # noqa: E402
    PINNED_SQLITE_VERSION,
)
from build_reference_db import build_reference_db as build_db  # noqa: E402
from extract_reference_schema import (  # type: ignore[import-not-found]  # noqa: E402
    extract_schema_sql,
)
from make_data_artifacts import (  # type: ignore[import-not-found]  # noqa: E402
    _fetch_sorted,
    _sha256,
    _table_layout,
    _write_csv,
    _write_manifest,
    export_table_parquet,
)
from verify_reference_roundtrip import (  # type: ignore[import-not-found]  # noqa: E402
    DatabaseNotFoundError,
    RoundtripReport,
    compare_databases,
    table_content_hash,
    view_content_hash,
)

pytestmark = pytest.mark.skipif(
    sqlite3.sqlite_version != PINNED_SQLITE_VERSION,
    reason=(
        f"byte-identity contract is defined only under the pinned SQLite "
        f"{PINNED_SQLITE_VERSION} (runtime: {sqlite3.sqlite_version}); the builder's "
        "version gate fires before any tested behavior -- CI runner pinning is a "
        "#46 Phase-6 cutover work item"
    ),
)


def _db(path: Path, statements: str) -> sqlite3.Connection:
    conn = sqlite3.connect(path)
    conn.executescript(statements)
    conn.commit()
    return conn


class TestIdenticalDatabasesAreOk:
    def test_identical_synthetic_dbs_are_ok(self, tmp_path: Path) -> None:
        schema = (
            "CREATE TABLE dim_k (id INTEGER PRIMARY KEY, name TEXT);\n"
            "CREATE VIEW view_k AS SELECT id, name FROM dim_k;\n"
        )
        live = _db(tmp_path / "live.sqlite", schema)
        live.executemany("INSERT INTO dim_k VALUES (?,?)", [(1, "a"), (2, "b")])
        live.commit()
        live.close()

        rebuilt = _db(tmp_path / "rebuilt.sqlite", schema)
        rebuilt.executemany("INSERT INTO dim_k VALUES (?,?)", [(1, "a"), (2, "b")])
        rebuilt.commit()
        rebuilt.close()

        report = compare_databases(tmp_path / "live.sqlite", tmp_path / "rebuilt.sqlite")
        assert isinstance(report, RoundtripReport)
        assert report.ok is True
        assert report.tables == {"dim_k": "ok"}
        assert report.views == {"view_k": "ok"}


class TestMutatedValueIsolatesTable:
    def test_mutation_flags_only_its_own_table(self, tmp_path: Path) -> None:
        schema = (
            "CREATE TABLE dim_a (id INTEGER PRIMARY KEY, name TEXT);\n"
            "CREATE TABLE dim_b (id INTEGER PRIMARY KEY, name TEXT);\n"
        )
        live = _db(tmp_path / "live.sqlite", schema)
        live.executemany("INSERT INTO dim_a VALUES (?,?)", [(1, "a"), (2, "b")])
        live.executemany("INSERT INTO dim_b VALUES (?,?)", [(1, "x"), (2, "y")])
        live.commit()
        live.close()

        rebuilt = _db(tmp_path / "rebuilt.sqlite", schema)
        rebuilt.executemany("INSERT INTO dim_a VALUES (?,?)", [(1, "a"), (2, "MUTATED")])
        rebuilt.executemany("INSERT INTO dim_b VALUES (?,?)", [(1, "x"), (2, "y")])
        rebuilt.commit()
        rebuilt.close()

        report = compare_databases(tmp_path / "live.sqlite", tmp_path / "rebuilt.sqlite")
        assert report.ok is False
        assert report.tables["dim_a"] == "content-hash mismatch"
        assert report.tables["dim_b"] == "ok"


class TestExtraRowIsRowCountMismatch:
    def test_extra_row_reported_as_row_count_mismatch(self, tmp_path: Path) -> None:
        schema = "CREATE TABLE dim_a (id INTEGER PRIMARY KEY, name TEXT);\n"
        live = _db(tmp_path / "live.sqlite", schema)
        live.executemany("INSERT INTO dim_a VALUES (?,?)", [(1, "a"), (2, "b")])
        live.commit()
        live.close()

        rebuilt = _db(tmp_path / "rebuilt.sqlite", schema)
        rebuilt.executemany("INSERT INTO dim_a VALUES (?,?)", [(1, "a"), (2, "b"), (3, "c")])
        rebuilt.commit()
        rebuilt.close()

        report = compare_databases(tmp_path / "live.sqlite", tmp_path / "rebuilt.sqlite")
        assert report.ok is False
        assert report.tables["dim_a"] == "row-count: 2 vs 3"


class TestNumericStorageClassCanonicalization:
    """The D2/NUMERIC rule: storage-class drift between INTEGER and REAL for
    the same logical value, across columns whose declared type maps to
    float64, is invisible to the comparison. SQLite's NUMERIC affinity always
    collapses a lossless integer-valued REAL down to INTEGER storage on
    insert, and REAL affinity always forces the opposite — so a NUMERIC- vs
    REAL-declared column is the natural way to construct genuine storage-class
    drift for the identical logical value 1/1.0 (verified empirically before
    writing this fixture)."""

    def test_integer_vs_real_storage_class_is_equal(self, tmp_path: Path) -> None:
        live = _db(
            tmp_path / "live.sqlite", "CREATE TABLE fact_m (id INTEGER PRIMARY KEY, v NUMERIC);\n"
        )
        live.execute("INSERT INTO fact_m VALUES (1, ?)", (1,))
        live.commit()
        assert live.execute("SELECT typeof(v) FROM fact_m").fetchone()[0] == "integer"
        live.close()

        rebuilt = _db(
            tmp_path / "rebuilt.sqlite", "CREATE TABLE fact_m (id INTEGER PRIMARY KEY, v REAL);\n"
        )
        rebuilt.execute("INSERT INTO fact_m VALUES (1, ?)", (1,))
        rebuilt.commit()
        assert rebuilt.execute("SELECT typeof(v) FROM fact_m").fetchone()[0] == "real"
        rebuilt.close()

        report = compare_databases(tmp_path / "live.sqlite", tmp_path / "rebuilt.sqlite")
        assert report.tables["fact_m"] == "ok"
        assert report.ok is True


class TestBooleanCanonicalization:
    """BOOLEAN-declared columns canonicalize non-0/1 truthiness to 0/1,
    mirroring the exporter's export-time ``bool(v)`` coercion."""

    def test_live_two_rebuilt_one_is_equal(self, tmp_path: Path) -> None:
        schema = "CREATE TABLE fact_flag (id INTEGER PRIMARY KEY, active BOOLEAN);\n"
        live = _db(tmp_path / "live.sqlite", schema)
        live.execute("INSERT INTO fact_flag VALUES (1, ?)", (2,))
        live.commit()
        live.close()

        rebuilt = _db(tmp_path / "rebuilt.sqlite", schema)
        rebuilt.execute("INSERT INTO fact_flag VALUES (1, ?)", (1,))
        rebuilt.commit()
        rebuilt.close()

        report = compare_databases(tmp_path / "live.sqlite", tmp_path / "rebuilt.sqlite")
        assert report.tables["fact_flag"] == "ok"
        assert report.ok is True

    def test_live_one_rebuilt_zero_is_mismatch(self, tmp_path: Path) -> None:
        schema = "CREATE TABLE fact_flag (id INTEGER PRIMARY KEY, active BOOLEAN);\n"
        live = _db(tmp_path / "live.sqlite", schema)
        live.execute("INSERT INTO fact_flag VALUES (1, ?)", (1,))
        live.commit()
        live.close()

        rebuilt = _db(tmp_path / "rebuilt.sqlite", schema)
        rebuilt.execute("INSERT INTO fact_flag VALUES (1, ?)", (0,))
        rebuilt.commit()
        rebuilt.close()

        report = compare_databases(tmp_path / "live.sqlite", tmp_path / "rebuilt.sqlite")
        assert report.tables["fact_flag"] == "content-hash mismatch"
        assert report.ok is False

    def test_null_stays_equal(self, tmp_path: Path) -> None:
        schema = "CREATE TABLE fact_flag (id INTEGER PRIMARY KEY, active BOOLEAN);\n"
        live = _db(tmp_path / "live.sqlite", schema)
        live.execute("INSERT INTO fact_flag VALUES (1, ?)", (None,))
        live.commit()
        live.close()

        rebuilt = _db(tmp_path / "rebuilt.sqlite", schema)
        rebuilt.execute("INSERT INTO fact_flag VALUES (1, ?)", (None,))
        rebuilt.commit()
        rebuilt.close()

        report = compare_databases(tmp_path / "live.sqlite", tmp_path / "rebuilt.sqlite")
        assert report.tables["fact_flag"] == "ok"


class TestPkLessTableSortsByAllColumns:
    def test_pkless_table_matches_across_insertion_order(self, tmp_path: Path) -> None:
        schema = "CREATE TABLE dim_note (code TEXT, note TEXT);\n"
        live = _db(tmp_path / "live.sqlite", schema)
        live.executemany("INSERT INTO dim_note VALUES (?,?)", [("a", "x"), ("b", "y")])
        live.commit()
        live.close()

        rebuilt = _db(tmp_path / "rebuilt.sqlite", schema)
        # Same rows, reversed physical insertion order.
        rebuilt.executemany("INSERT INTO dim_note VALUES (?,?)", [("b", "y"), ("a", "x")])
        rebuilt.commit()
        rebuilt.close()

        report = compare_databases(tmp_path / "live.sqlite", tmp_path / "rebuilt.sqlite")
        assert report.tables["dim_note"] == "ok"
        assert report.ok is True


class TestMissingObjects:
    def test_table_missing_in_rebuilt(self, tmp_path: Path) -> None:
        live = _db(tmp_path / "live.sqlite", "CREATE TABLE dim_a (id INTEGER PRIMARY KEY);\n")
        live.close()
        rebuilt = _db(tmp_path / "rebuilt.sqlite", "")
        rebuilt.close()

        report = compare_databases(tmp_path / "live.sqlite", tmp_path / "rebuilt.sqlite")
        assert report.tables["dim_a"] == "missing-in-rebuilt"
        assert report.ok is False

    def test_table_missing_in_live(self, tmp_path: Path) -> None:
        live = _db(tmp_path / "live.sqlite", "")
        live.close()
        rebuilt = _db(tmp_path / "rebuilt.sqlite", "CREATE TABLE dim_a (id INTEGER PRIMARY KEY);\n")
        rebuilt.close()

        report = compare_databases(tmp_path / "live.sqlite", tmp_path / "rebuilt.sqlite")
        assert report.tables["dim_a"] == "missing-in-live"
        assert report.ok is False

    def test_view_mismatch_reported_independently_of_tables(self, tmp_path: Path) -> None:
        schema = (
            "CREATE TABLE dim_a (id INTEGER PRIMARY KEY, v INTEGER);\n"
            "CREATE VIEW view_a AS SELECT id, v * 2 AS doubled FROM dim_a;\n"
        )
        live = _db(tmp_path / "live.sqlite", schema)
        live.execute("INSERT INTO dim_a VALUES (1, 5)")
        live.commit()
        live.close()

        rebuilt = _db(tmp_path / "rebuilt.sqlite", schema.replace("v * 2", "v * 3"))
        rebuilt.execute("INSERT INTO dim_a VALUES (1, 5)")
        rebuilt.commit()
        rebuilt.close()

        report = compare_databases(tmp_path / "live.sqlite", tmp_path / "rebuilt.sqlite")
        assert report.tables["dim_a"] == "ok"  # underlying data identical
        assert report.views["view_a"] == "content-hash mismatch"  # derived value differs
        assert report.ok is False


class TestSqliteInternalTablesExcluded:
    def test_sqlite_sequence_excluded(self, tmp_path: Path) -> None:
        schema = "CREATE TABLE dim_a (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT);\n"
        live = _db(tmp_path / "live.sqlite", schema)
        live.execute("INSERT INTO dim_a (name) VALUES ('x')")  # materializes sqlite_sequence
        live.commit()
        live.close()

        rebuilt = _db(tmp_path / "rebuilt.sqlite", schema)
        rebuilt.execute("INSERT INTO dim_a (name) VALUES ('x')")
        rebuilt.commit()
        rebuilt.close()

        report = compare_databases(tmp_path / "live.sqlite", tmp_path / "rebuilt.sqlite")
        assert "sqlite_sequence" not in report.tables
        assert report.ok is True


class TestOpensReadOnly:
    def test_connections_opened_mode_ro(
        self, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        live_path = tmp_path / "live.sqlite"
        rebuilt_path = tmp_path / "rebuilt.sqlite"
        _db(live_path, "CREATE TABLE dim_a (id INTEGER PRIMARY KEY);\n").close()
        _db(rebuilt_path, "CREATE TABLE dim_a (id INTEGER PRIMARY KEY);\n").close()

        seen_uris: list[str] = []
        real_connect = sqlite3.connect

        def _spy_connect(database: str, **kwargs: object) -> sqlite3.Connection:
            seen_uris.append(database)
            return real_connect(database, **kwargs)  # type: ignore[arg-type]

        monkeypatch.setattr(verify_reference_roundtrip.sqlite3, "connect", _spy_connect)
        compare_databases(live_path, rebuilt_path)

        assert len(seen_uris) == 2
        assert all(uri.startswith("file:") and uri.endswith("?mode=ro") for uri in seen_uris)


class TestDatabaseNotFound:
    def test_missing_live_is_fatal(self, tmp_path: Path) -> None:
        rebuilt_path = tmp_path / "rebuilt.sqlite"
        _db(rebuilt_path, "CREATE TABLE dim_a (id INTEGER PRIMARY KEY);\n").close()
        with pytest.raises(DatabaseNotFoundError, match="live"):
            compare_databases(tmp_path / "nope.sqlite", rebuilt_path)

    def test_missing_rebuilt_is_fatal(self, tmp_path: Path) -> None:
        live_path = tmp_path / "live.sqlite"
        _db(live_path, "CREATE TABLE dim_a (id INTEGER PRIMARY KEY);\n").close()
        with pytest.raises(DatabaseNotFoundError, match="rebuilt"):
            compare_databases(live_path, tmp_path / "nope.sqlite")


class TestCliExitContract:
    def test_cli_exits_zero_when_ok(
        self, tmp_path: Path, capsys: pytest.CaptureFixture[str]
    ) -> None:
        schema = "CREATE TABLE dim_a (id INTEGER PRIMARY KEY, name TEXT);\n"
        _db(tmp_path / "live.sqlite", schema).close()
        _db(tmp_path / "rebuilt.sqlite", schema).close()

        rc = verify_reference_roundtrip.main(
            ["--live", str(tmp_path / "live.sqlite"), "--rebuilt", str(tmp_path / "rebuilt.sqlite")]
        )
        assert rc == 0
        out = capsys.readouterr().out
        assert "ok=True" in out

    def test_cli_exits_one_and_prints_mismatch_on_drift(
        self, tmp_path: Path, capsys: pytest.CaptureFixture[str]
    ) -> None:
        schema = "CREATE TABLE dim_a (id INTEGER PRIMARY KEY, name TEXT);\n"
        live = _db(tmp_path / "live.sqlite", schema)
        live.execute("INSERT INTO dim_a VALUES (1, 'a')")
        live.commit()
        live.close()

        rebuilt = _db(tmp_path / "rebuilt.sqlite", schema)
        rebuilt.execute("INSERT INTO dim_a VALUES (1, 'MUTATED')")
        rebuilt.commit()
        rebuilt.close()

        rc = verify_reference_roundtrip.main(
            ["--live", str(tmp_path / "live.sqlite"), "--rebuilt", str(tmp_path / "rebuilt.sqlite")]
        )
        assert rc == 1
        captured = capsys.readouterr()
        assert "dim_a" in captured.err
        assert "content-hash mismatch" in captured.err
        assert "ok=False" in captured.out

    def test_cli_missing_db_is_loud(self, tmp_path: Path) -> None:
        _db(tmp_path / "rebuilt.sqlite", "").close()
        with pytest.raises(DatabaseNotFoundError):
            verify_reference_roundtrip.main(
                [
                    "--live",
                    str(tmp_path / "nope.sqlite"),
                    "--rebuilt",
                    str(tmp_path / "rebuilt.sqlite"),
                ]
            )


class TestDirectHashHelpers:
    """Direct coverage of the exported ``table_content_hash``/
    ``view_content_hash`` interfaces the plan names explicitly."""

    def test_table_content_hash_row_count_and_determinism(self, tmp_path: Path) -> None:
        conn = _db(
            tmp_path / "db.sqlite", "CREATE TABLE dim_a (id INTEGER PRIMARY KEY, name TEXT);\n"
        )
        conn.executemany("INSERT INTO dim_a VALUES (?,?)", [(1, "a"), (2, "b")])
        conn.commit()
        count1, hash1 = table_content_hash(conn, "dim_a")
        count2, hash2 = table_content_hash(conn, "dim_a")
        conn.close()
        assert count1 == 2
        assert hash1 == hash2

    def test_view_content_hash_row_count(self, tmp_path: Path) -> None:
        conn = _db(
            tmp_path / "db.sqlite",
            "CREATE TABLE dim_a (id INTEGER PRIMARY KEY, v INTEGER);\n"
            "CREATE VIEW view_a AS SELECT id, v FROM dim_a;\n",
        )
        conn.executemany("INSERT INTO dim_a VALUES (?,?)", [(1, 10), (2, 20)])
        conn.commit()
        count, _digest = view_content_hash(conn, "view_a")
        conn.close()
        assert count == 2


class TestFullPipelineRoundTrip:
    """The first full-pipeline synthetic round-trip proof: export through the
    REAL exporter (parquet + CSV tiers), extract schema.sql through the REAL
    extractor, build through the REAL builder, then verify the rebuilt
    product content-matches its source through this module's own
    ``compare_databases`` — end to end, nothing mocked."""

    def test_source_matches_real_builder_output(self, tmp_path: Path) -> None:
        source_path = tmp_path / "src.sqlite"
        conn = sqlite3.connect(source_path)
        conn.executescript(
            # Canonical contiguous-block DDL shape (tables, then indexes,
            # then views) — the builder's replay contract hard-fails
            # interleavings.
            "CREATE TABLE dim_k (id INTEGER PRIMARY KEY, name TEXT);\n"
            "CREATE TABLE fact_m (id INTEGER PRIMARY KEY, k_id INTEGER, v NUMERIC);\n"
            "CREATE TABLE fact_flag (id INTEGER PRIMARY KEY, active BOOLEAN, seen_on DATE);\n"
            "CREATE TABLE dim_note (id INTEGER PRIMARY KEY, code TEXT, note TEXT, weight NUMERIC);\n"
            "CREATE INDEX idx_fact_m_k ON fact_m (k_id);\n"
            "CREATE VIEW view_m AS SELECT k_id, SUM(v) s FROM fact_m GROUP BY k_id;\n"
        )
        conn.executemany("INSERT INTO dim_k VALUES (?,?)", [(1, "a"), (2, "b")])
        conn.executemany("INSERT INTO fact_m VALUES (?,?,?)", [(1, 1, 2.5), (2, 2, 4.0)])
        conn.executemany(
            "INSERT INTO fact_flag VALUES (?,?,?)",
            [(1, True, "2024-01-01"), (2, False, None)],
        )
        conn.executemany(
            "INSERT INTO dim_note VALUES (?,?,?,?)",
            # Note: no NULL in a TEXT column here (unlike
            # test_build_reference_db.py's fixture) — the CSV tier's
            # documented lossy edge (a real NULL in a TEXT column becomes ""
            # on rebuild, since the csv module serializes None and ""
            # identically) is a pre-existing, ACCEPTED builder transformation,
            # not something this verifier canonicalizes away. Keeping it out
            # of this fixture lets the assertion below be a clean ok=True
            # proof of genuine content equality. A NULL in the NUMERIC
            # `weight` column round-trips losslessly (non-text: "" -> None on
            # read) so it stays in to exercise that path.
            [(1, "x", "", 1.5), (2, "y", "yy", None)],
        )
        conn.commit()

        csv_tables = frozenset({"dim_note"})
        all_tables = ("dim_k", "fact_m", "fact_flag", "dim_note")
        entries: list[dict[str, object]] = []
        for table in all_tables:
            columns, pk, _schema = _table_layout(conn, table)
            if table in csv_tables:
                home = tmp_path / f"{table}.csv"
                data = _fetch_sorted(conn, table, columns, pk)
                _write_csv(home, columns, data)
                fmt = "csv"
                rows = len(data)
            else:
                home = tmp_path / f"{table}.parquet"
                rows, _size = export_table_parquet(conn, table, home)
                fmt = "parquet"
            entries.append(
                {
                    "name": table,
                    "format": fmt,
                    "source_table": table,
                    "mode": "generate",
                    "rows": rows,
                    "sha256": _sha256(home),
                    "home": str(home.relative_to(tmp_path)),
                    "material_relation": f"synthetic fixture table {table}",
                }
            )
        schema_path = tmp_path / "schema.sql"
        schema_path.write_text(extract_schema_sql(conn))
        conn.close()

        manifest_path = tmp_path / "data-artifacts.yaml"
        _write_manifest(entries, path=manifest_path)

        rebuilt_path = tmp_path / "rebuilt.sqlite"
        result = build_db(manifest_path, schema_path, tmp_path, rebuilt_path)
        assert result.path == rebuilt_path

        report = compare_databases(source_path, rebuilt_path)
        assert report.ok is True, report
        assert set(report.tables) == set(all_tables)
        assert all(v == "ok" for v in report.tables.values())
        assert report.views == {"view_m": "ok"}
