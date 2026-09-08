"""Task 5: the deterministic reference-DB builder — double-build byte-identity
proof, content round trip, and the four hard-fail paths (parquet pipeline plan
2026-07-19).

All fixtures are synthetic temp SQLite databases; the real reference DB is
never touched. The shared ``_make_source_db`` fixture is a superset of the
plan's literal example (``dim_k``/``fact_m``/``idx_fact_m_k``/``view_m``) that
also carries a BOOLEAN + DATE column (``fact_flag``, parquet tier) and a
CSV-tier table (``dim_note``) so the double-build byte-identity proof and the
content round trip both exercise every load path the builder has (parquet
batch streaming, CSV NULL/type coercion, index/view replay) in one pass.
"""

from __future__ import annotations

import sqlite3
import sys
from pathlib import Path

import pytest
import yaml

_REPO_ROOT = Path(__file__).resolve().parents[3]
TOOLS_DIR = _REPO_ROOT / "tools"
sys.path.insert(0, str(TOOLS_DIR))

import build_reference_db  # type: ignore[import-not-found]  # noqa: E402
from build_reference_db import (  # type: ignore[import-not-found]  # noqa: E402
    APPLICATION_ID,
    PINNED_SQLITE_VERSION,
    USER_VERSION,
    BuildEnvironmentError,
    BuildResult,
    CoverageHoleError,
    SchemaParseError,
    SourceHashError,
)
from build_reference_db import build_reference_db as build_db  # noqa: E402
from extract_reference_schema import (  # type: ignore[import-not-found]  # noqa: E402
    extract_schema_sql,
    schema_census,
)
from make_data_artifacts import (  # type: ignore[import-not-found]  # noqa: E402
    _fetch_sorted,
    _sha256,
    _table_layout,
    _write_csv,
    _write_manifest,
    export_table_parquet,
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

_CSV_TABLES = frozenset({"dim_note"})
_ALL_TABLES = ("dim_k", "fact_m", "fact_flag", "dim_note")


def _make_source_db(tmp_path: Path) -> sqlite3.Connection:
    """The plan's literal fixture (``dim_k``/``fact_m``/index/view) plus a
    BOOLEAN+DATE parquet table and a CSV-tier table."""
    conn = sqlite3.connect(tmp_path / "src.sqlite")
    conn.executescript(
        # Canonical contiguous-block DDL shape (tables, then indexes, then
        # views) — the builder's replay contract hard-fails interleavings.
        "CREATE TABLE dim_k (id INTEGER PRIMARY KEY, name TEXT);\n"
        "CREATE TABLE fact_m (id INTEGER PRIMARY KEY, k_id INTEGER, v REAL);\n"
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
        [(1, "x", "", 1.5), (2, "y", None, None)],
    )
    conn.commit()
    return conn


def _export_all(conn: sqlite3.Connection, tmp_path: Path) -> tuple[Path, Path]:
    """Export every table in ``conn`` through the REAL exporter/extractor/
    manifest-writer: ``dim_note`` goes through the CSV tier (in-repo-shaped),
    everything else through parquet — so the builder's two load paths are
    both exercised by every test that uses this fixture. Returns
    ``(manifest_path, schema_path)``, both under ``tmp_path`` (the caller
    passes ``tmp_path`` as ``sources_root`` too, since every ``home`` entry
    below is written relative to it)."""
    entries: list[dict[str, object]] = []
    for table in _ALL_TABLES:
        columns, pk, _schema = _table_layout(conn, table)
        if table in _CSV_TABLES:
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
    census = schema_census(conn)
    schema_entry: dict[str, object] = {
        "file": str(schema_path.relative_to(tmp_path)),
        "sha256": _sha256(schema_path),
        **census,
    }
    manifest_path = tmp_path / "data-artifacts.yaml"
    _write_manifest(entries, schema_entry=schema_entry, path=manifest_path)
    return manifest_path, schema_path


class TestDoubleBuildByteIdentity:
    def test_double_build_is_byte_identical(self, tmp_path: Path) -> None:
        conn = _make_source_db(tmp_path)
        try:
            manifest, schema = _export_all(conn, tmp_path)
        finally:
            conn.close()
        r1 = build_db(manifest, schema, tmp_path, tmp_path / "out1.sqlite")
        r2 = build_db(manifest, schema, tmp_path, tmp_path / "out2.sqlite")
        assert r1.sha256 == r2.sha256
        assert isinstance(r1, BuildResult)
        assert r1.sqlite_version == PINNED_SQLITE_VERSION

    def test_host_secure_delete_default_does_not_change_product_bytes(
        self, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        conn = _make_source_db(tmp_path)
        try:
            conn.executemany(
                "INSERT INTO dim_k VALUES (?, ?)",
                [(i + 3, "x" * ((i * 31) % 713)) for i in range(5000)],
            )
            conn.commit()
            manifest, schema = _export_all(conn, tmp_path)
        finally:
            conn.close()

        original_connect = sqlite3.connect
        final_settings: list[int] = []

        class ObservedConnection(sqlite3.Connection):
            def close(self) -> None:
                final_settings.append(self.execute("PRAGMA secure_delete").fetchone()[0])
                super().close()

        hashes: list[str] = []
        for default in (0, 1):

            def connect(path: Path, ambient_default: int = default) -> sqlite3.Connection:
                connection = original_connect(path, factory=ObservedConnection)
                connection.execute(f"PRAGMA secure_delete={ambient_default}")
                return connection

            monkeypatch.setattr(build_reference_db.sqlite3, "connect", connect)
            result = build_db(manifest, schema, tmp_path, tmp_path / f"out-{default}.sqlite")
            hashes.append(result.sha256)

        assert hashes[0] == hashes[1]
        assert final_settings == [1, 1]


class TestSchemaOrderFidelity:
    def test_index_rowid_order_survives_manifest_order_divergence(self, tmp_path: Path) -> None:
        # Cutover Step 3 regression (2026-07-20): the real DB's sqlite_master
        # carries indexes in historical creation order while manifest entries
        # are name-sorted; creating each table's indexes per-load (manifest
        # order) broke the schema fixed point. Creation order here (fact_z
        # before fact_a, indexes interleaved) deliberately inverts the
        # manifest's alphabetical order to pin the law: extract(source) ==
        # extract(rebuilt), byte for byte.
        # Same shape as the real schema.sql (all tables, then all indexes,
        # then views — verified TABLE*75/INDEX*99/VIEW*8 at cutover): the
        # builder's replay contract requires that shape and hard-fails on
        # interleavings, so the fixture keeps the blocks contiguous while
        # inverting alphabetical order inside each.
        conn = sqlite3.connect(tmp_path / "src.sqlite")
        conn.executescript(
            "CREATE TABLE fact_z (id INTEGER PRIMARY KEY, v REAL);\n"
            "CREATE TABLE fact_a (id INTEGER PRIMARY KEY, w REAL);\n"
            "CREATE INDEX idx_fact_z_v ON fact_z (v);\n"
            "CREATE INDEX idx_fact_a_w ON fact_a (w);\n"
        )
        conn.execute("INSERT INTO fact_z VALUES (1, 1.0)")
        conn.execute("INSERT INTO fact_a VALUES (1, 2.0)")
        conn.commit()

        entries: list[dict[str, object]] = []
        for table in ("fact_a", "fact_z"):  # name-sorted, inverting creation order
            home = tmp_path / f"{table}.parquet"
            rows, _size = export_table_parquet(conn, table, home)
            entries.append(
                {
                    "name": table,
                    "format": "parquet",
                    "source_table": table,
                    "mode": "generate",
                    "rows": rows,
                    "sha256": _sha256(home),
                    "home": str(home.relative_to(tmp_path)),
                    "material_relation": f"synthetic fixture table {table}",
                }
            )
        source_schema = extract_schema_sql(conn)
        schema_path = tmp_path / "schema.sql"
        schema_path.write_text(source_schema)
        census = schema_census(conn)
        conn.close()
        manifest_path = tmp_path / "data-artifacts.yaml"
        _write_manifest(
            entries,
            schema_entry={
                "file": str(schema_path.relative_to(tmp_path)),
                "sha256": _sha256(schema_path),
                **census,
            },
            path=manifest_path,
        )

        result = build_db(manifest_path, schema_path, tmp_path, tmp_path / "out.sqlite")
        rebuilt = sqlite3.connect(result.path)
        try:
            rebuilt_schema = extract_schema_sql(rebuilt)
        finally:
            rebuilt.close()
        assert rebuilt_schema == source_schema


class TestBuildContentRoundTrip:
    def test_build_matches_source_content(self, tmp_path: Path) -> None:
        conn = _make_source_db(tmp_path)
        try:
            manifest, schema = _export_all(conn, tmp_path)
        finally:
            conn.close()
        r = build_db(manifest, schema, tmp_path, tmp_path / "out.sqlite")
        out = sqlite3.connect(r.path)
        try:
            assert out.execute("SELECT * FROM dim_k ORDER BY id").fetchall() == [
                (1, "a"),
                (2, "b"),
            ]
            assert out.execute("SELECT s FROM view_m ORDER BY k_id").fetchall() == [
                (2.5,),
                (4.0,),
            ]
            assert out.execute("PRAGMA application_id").fetchone()[0] == APPLICATION_ID
            assert out.execute("PRAGMA user_version").fetchone()[0] == USER_VERSION
            # BOOLEAN + DATE round trip through the parquet path: sqlite has
            # no native boolean storage class, so both the source and the
            # rebuilt DB read BOOLEAN columns back as plain 0/1 ints — never
            # Python bool objects (this codebase never opens sqlite3 with
            # detect_types=PARSE_DECLTYPES).
            assert out.execute(
                "SELECT id, active, seen_on FROM fact_flag ORDER BY id"
            ).fetchall() == [(1, 1, "2024-01-01"), (2, 0, None)]
            # CSV-tier round trip: TEXT columns keep an empty field as "" —
            # even when the true source value was NULL (row 2's `note`,
            # documented lossy edge: the csv module serializes None and ""
            # identically, so a TEXT column can't tell them apart on the way
            # back and always resolves to ""). Non-TEXT columns (NUMERIC
            # `weight`) treat an empty field as NULL instead, since a numeric
            # parse of "" isn't meaningful — this preserves row 2's NULL.
            assert out.execute(
                "SELECT id, code, note, weight FROM dim_note ORDER BY id"
            ).fetchall() == [(1, "x", "", 1.5), (2, "y", "", None)]
        finally:
            out.close()

    def test_rows_inserted_and_tables_built_counts(self, tmp_path: Path) -> None:
        conn = _make_source_db(tmp_path)
        try:
            manifest, schema = _export_all(conn, tmp_path)
        finally:
            conn.close()
        r = build_db(manifest, schema, tmp_path, tmp_path / "out.sqlite")
        assert r.tables_built == len(_ALL_TABLES)
        assert r.rows_inserted == 2 + 2 + 2 + 2  # dim_k, fact_m, fact_flag, dim_note


class TestSourceHashMismatchIsFatal:
    def test_corrupted_parquet_source_is_fatal(self, tmp_path: Path) -> None:
        conn = _make_source_db(tmp_path)
        try:
            manifest, schema = _export_all(conn, tmp_path)
        finally:
            conn.close()
        target = tmp_path / "dim_k.parquet"
        corrupted = bytearray(target.read_bytes())
        corrupted[-1] ^= 0xFF
        target.write_bytes(bytes(corrupted))
        with pytest.raises(SourceHashError, match="dim_k"):
            build_db(manifest, schema, tmp_path, tmp_path / "out.sqlite")

    def test_corrupted_csv_source_is_fatal(self, tmp_path: Path) -> None:
        conn = _make_source_db(tmp_path)
        try:
            manifest, schema = _export_all(conn, tmp_path)
        finally:
            conn.close()
        target = tmp_path / "dim_note.csv"
        target.write_text(target.read_text() + "9,z,zz,9.9\n")
        with pytest.raises(SourceHashError, match="dim_note"):
            build_db(manifest, schema, tmp_path, tmp_path / "out.sqlite")


class TestSchemaTableWithoutSourceIsFatal:
    def test_dropped_manifest_entry_is_fatal(self, tmp_path: Path) -> None:
        conn = _make_source_db(tmp_path)
        try:
            manifest, schema = _export_all(conn, tmp_path)
        finally:
            conn.close()
        parsed = yaml.safe_load(manifest.read_text())
        remaining = [e for e in parsed["artifacts"] if e["source_table"] != "dim_k"]
        _write_manifest(remaining, schema_entry=parsed.get("schema"), path=manifest)
        with pytest.raises(CoverageHoleError, match="dim_k"):
            build_db(manifest, schema, tmp_path, tmp_path / "out.sqlite")


class TestWrongSqliteVersionIsFatal:
    def test_version_mismatch_is_fatal(
        self, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        conn = _make_source_db(tmp_path)
        try:
            manifest, schema = _export_all(conn, tmp_path)
        finally:
            conn.close()
        monkeypatch.setattr(build_reference_db.sqlite3, "sqlite_version", "9.9.9")
        with pytest.raises(BuildEnvironmentError, match="9.9.9"):
            build_db(manifest, schema, tmp_path, tmp_path / "out.sqlite")


class TestBuildEnvironmentErrors:
    def test_missing_manifest_is_fatal(self, tmp_path: Path) -> None:
        conn = _make_source_db(tmp_path)
        try:
            _manifest, schema = _export_all(conn, tmp_path)
        finally:
            conn.close()
        with pytest.raises(BuildEnvironmentError, match="manifest"):
            build_db(tmp_path / "nope.yaml", schema, tmp_path, tmp_path / "out.sqlite")

    def test_missing_schema_sql_is_fatal(self, tmp_path: Path) -> None:
        conn = _make_source_db(tmp_path)
        try:
            manifest, _schema = _export_all(conn, tmp_path)
        finally:
            conn.close()
        with pytest.raises(BuildEnvironmentError, match="schema"):
            build_db(manifest, tmp_path / "nope.sql", tmp_path, tmp_path / "out.sqlite")

    def test_missing_source_file_is_fatal(self, tmp_path: Path) -> None:
        conn = _make_source_db(tmp_path)
        try:
            manifest, schema = _export_all(conn, tmp_path)
        finally:
            conn.close()
        (tmp_path / "dim_k.parquet").unlink()
        with pytest.raises(BuildEnvironmentError, match="dim_k"):
            build_db(manifest, schema, tmp_path, tmp_path / "out.sqlite")


class TestSchemaParsing:
    """Directly exercises the statement classifier — the plan's determinism
    decision 3 (unclassifiable statement / unmatched index target are both
    hard errors, never silently skipped)."""

    def test_unclassifiable_statement_is_fatal(self, tmp_path: Path) -> None:
        schema_path = tmp_path / "schema.sql"
        schema_path.write_text("PRAGMA foreign_keys=ON;\n")
        with pytest.raises(SchemaParseError):
            list(build_reference_db._read_schema_statements(schema_path))

    def test_index_with_no_resolvable_target_is_fatal(self, tmp_path: Path) -> None:
        schema_path = tmp_path / "schema.sql"
        # Deliberately malformed CREATE INDEX (no "ON <table>") to hit the
        # plan's specified hard-error path for an unmatched index target.
        schema_path.write_text(
            "CREATE TABLE fact_a (id INTEGER PRIMARY KEY);\n\nCREATE INDEX idx_bad;\n"
        )
        with pytest.raises(SchemaParseError):
            list(build_reference_db._read_schema_statements(schema_path))


class TestCoerceCsvCell:
    """Direct unit coverage of :func:`build_reference_db._coerce_csv_cell` —
    the CSV-tier per-cell coercion function (Tasks 5/6/7 review fold-ins 2+3).
    """

    def test_boolean_decltype_coerces_via_bool_int(self) -> None:
        assert build_reference_db._coerce_csv_cell("1", "BOOLEAN", "active") is True
        assert build_reference_db._coerce_csv_cell("0", "BOOLEAN", "active") is False

    def test_unknown_decltype_is_fatal(self) -> None:
        with pytest.raises(SchemaParseError, match="mystery_col.*BLOB"):
            build_reference_db._coerce_csv_cell("x", "BLOB", "mystery_col")


class TestCliEndToEnd:
    def test_cli_writes_db_and_updates_manifest(self, tmp_path: Path) -> None:
        conn = _make_source_db(tmp_path)
        try:
            manifest, schema = _export_all(conn, tmp_path)
        finally:
            conn.close()
        out = tmp_path / "cli_out.sqlite"
        rc = build_reference_db.main(
            [
                "--manifest",
                str(manifest),
                "--schema",
                str(schema),
                "--sources-root",
                str(tmp_path),
                "--out",
                str(out),
                "--update-manifest",
            ]
        )
        assert rc == 0
        assert out.is_file()
        parsed = yaml.safe_load(manifest.read_text())
        assert parsed["product"]["sha256"] == _sha256(out)
        assert parsed["product"]["page_size"] == build_reference_db.PAGE_SIZE
        assert parsed["product"]["application_id"] == APPLICATION_ID
        assert parsed["product"]["user_version"] == USER_VERSION
        assert parsed["product"]["sqlite_version"] == PINNED_SQLITE_VERSION
        # artifacts/schema untouched by the manifest update
        assert parsed["schema"]["sha256"] == _sha256(schema)

    def test_cli_missing_out_is_a_usage_error(self, tmp_path: Path) -> None:
        with pytest.raises(SystemExit):
            build_reference_db.main(["--manifest", str(tmp_path / "nope.yaml")])
