#!/usr/bin/env python3
"""Deterministic reference-DB builder for the parquet-canonical pipeline (plan
2026-07-19, Task 5).

Rebuilds ``marxist-data-3NF.sqlite`` from three canonical sources — the
manifest (``data-artifacts.yaml``, Task 3), the DDL (``schema.sql``, Task 4),
and the per-table parquet/CSV artifacts themselves (Tasks 1-2) — byte for
byte, twice in a row, with nothing but those three inputs and a pinned SQLite
runtime. This is the authority inversion the plan is for: the parquet sources
+ schema.sql become canonical, and the SQLite file becomes a build product.

Build algorithm (every point is a fixed determinism decision):

1. Assert the runtime ``sqlite3.sqlite_version`` matches :data:`PINNED_SQLITE_VERSION`.
2. Fresh output file (unlink first); ``PRAGMA page_size``, ``PRAGMA
   journal_mode=DELETE`` before any DDL.
3. Split ``schema.sql`` into statements, classify each by leading keyword
   (``CREATE TABLE`` / ``CREATE [UNIQUE] INDEX`` / ``CREATE VIEW`` /
   ``CREATE TRIGGER``) — an unclassifiable statement is a hard error. An
   index statement's target table comes from ``... ON <table>``; no match is
   a hard error.
4. Execute all ``CREATE TABLE`` statements, in file order.
5. For every manifest artifact entry whose ``source_table`` has a
   ``CREATE TABLE`` in ``schema.sql`` (this membership test is what
   distinguishes DB sources from artifact-only demoted tables), in manifest
   order: verify the source file's sha256 against the manifest (hard error on
   mismatch), stream rows in file order, ``executemany`` INSERT, then execute
   that table's ``CREATE INDEX`` statements in file order. A schema table
   with NO manifest entry is a hard error (a coverage hole); the reverse (a
   manifest entry whose table has no schema.sql source, e.g. a
   already-demoted ``register``-mode artifact) is simply skipped — already
   covered by the membership test.
6. Execute ``CREATE VIEW`` (and any ``CREATE TRIGGER``) statements, in file
   order.
7. ``PRAGMA application_id``, ``PRAGMA user_version``; commit; ``VACUUM``;
   close.
8. Return a :class:`BuildResult` with the finished file's sha256.

**Trigger support is aspirational, not exercised.** Step 3's statement
splitter (:func:`_split_statements`) cuts ``schema.sql`` on a bare ``;\\n``
terminator, which assumes no statement contains an un-terminated ``;`` of
its own. A ``CREATE TRIGGER ... BEGIN ... END`` body with internal
``;``-terminated statements breaks that assumption and is NOT correctly
split. The real reference DB has 0 triggers today (``schema_census``
confirms), so this never fires in practice; if a future migration adds one,
the intended failure mode is a loud :class:`SchemaParseError` (the malformed
fragment fails to classify, or SQLite rejects it at execute time) — never
silent truncation (Constitution III.11). A trigger-bearing schema needs a
smarter splitter before this builder can claim real trigger support.

Usage::

    uv run python tools/build_reference_db.py --out dist/build/ref.sqlite
    uv run python tools/build_reference_db.py --out dist/build/ref.sqlite --update-manifest
"""

from __future__ import annotations

import argparse
import csv
import dataclasses
import re
import sqlite3
import sys
from pathlib import Path
from typing import Literal

import pyarrow.parquet as pq  # type: ignore[import-untyped]

sys.path.insert(0, str(Path(__file__).resolve().parent))
from make_data_artifacts import (  # noqa: E402
    MANIFEST_PATH,
    _sha256,
    update_product_block,
)

#: Measured on the live reference DB (Phase 0 report). Changing any of the
#: four pins below is a DECLARED regeneration event (plan Global Constraints).
PAGE_SIZE = 4096
APPLICATION_ID = 0x4241424C  # "BABL"
USER_VERSION = 1  # pipeline major version
PINNED_SQLITE_VERSION = "3.53.1"  # Must match data-artifacts.yaml product.sqlite_version.
#: mise.lock pins the native Python build containing this SQLite version;
#: use `mise exec -- uv run --frozen python ...` for reference reproduction. Bumped
#: pre-cutover: no build products were ever minted under 3.46.1, so no regeneration
#: event is owed (the plan's "bump = declared regeneration event" rule is vacuous here).

#: Default canonical locations (repo-root-relative), matching the sibling
#: exporters' ``_DEFAULT_DB``/``DEFAULT_OUT`` precedent.
DEFAULT_MANIFEST = MANIFEST_PATH
DEFAULT_SCHEMA = Path("dist/data-artifacts/schema.sql")
DEFAULT_SOURCES_ROOT = Path()


class ReferenceBuildError(Exception):
    """Base for every builder failure — callers never catch this generically,
    only one of the specific subclasses below (Constitution III.11)."""


class BuildEnvironmentError(ReferenceBuildError):
    """The build environment itself is wrong: unpinned SQLite runtime, or a
    required input file (manifest, schema.sql, a manifest-declared source
    file) is missing."""


class SourceHashError(ReferenceBuildError):
    """A manifest-declared source file's sha256 doesn't match the manifest's
    recorded hash — the source drifted or was corrupted since export."""


class CoverageHoleError(ReferenceBuildError):
    """A ``schema.sql`` ``CREATE TABLE`` has no corresponding manifest
    artifact entry — the table would build empty and silently."""


class SchemaParseError(ReferenceBuildError):
    """``schema.sql`` contains a statement the builder cannot classify, or an
    index statement whose target table cannot be resolved."""


@dataclasses.dataclass(frozen=True)
class BuildResult:
    """The finished build product's identity."""

    path: Path
    sha256: str
    sqlite_version: str
    tables_built: int
    rows_inserted: int


@dataclasses.dataclass(frozen=True)
class _Statement:
    """One classified ``schema.sql`` statement."""

    kind: Literal["table", "index", "view", "trigger"]
    table: str | None
    sql: str


_CREATE_TABLE_RE = re.compile(r"^\s*CREATE\s+TABLE\b", re.IGNORECASE)
_CREATE_INDEX_RE = re.compile(r"^\s*CREATE\s+(?:UNIQUE\s+)?INDEX\b", re.IGNORECASE)
_CREATE_VIEW_RE = re.compile(r"^\s*CREATE\s+VIEW\b", re.IGNORECASE)
_CREATE_TRIGGER_RE = re.compile(r"^\s*CREATE\s+TRIGGER\b", re.IGNORECASE)
_TABLE_NAME_RE = re.compile(r'CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?"?(\w+)"?', re.IGNORECASE)
#: Plan-pinned regex for an index statement's target table (Task 5 step 3).
_INDEX_TABLE_RE = re.compile(r'\bON\s+"?(\w+)"?', re.IGNORECASE)

#: Declared-type prefixes whose CSV round trip keeps an empty field as an
#: empty string, never NULL — see :func:`_coerce_csv_cell`.
_TEXT_LIKE_DECLTYPE_PREFIXES = ("TEXT", "VARCHAR")


def _split_statements(schema_sql_text: str) -> list[str]:
    """Recover individual statements from ``schema.sql``'s canonical form
    (:func:`extract_reference_schema.extract_schema_sql`): comment lines
    (the header) stripped first, then split on the statement terminator the
    extractor guarantees — exactly one trailing ``;`` per statement, one
    blank line between statements.

    .. warning::
       This split is NOT safe for a ``CREATE TRIGGER ... BEGIN ... END`` body
       that contains its own ``;``-terminated statements — the trigger body
       gets cut at the wrong points. See the module docstring's "Trigger
       support is aspirational" note: the real DB has 0 triggers today, and
       the fallout of a future one is a loud :class:`SchemaParseError`, never
       silent truncation.

    :param schema_sql_text: Full ``schema.sql`` file contents.
    :returns: Statement bodies (no trailing ``;``), in file order.
    """
    body_lines = [
        line for line in schema_sql_text.splitlines() if not line.lstrip().startswith("--")
    ]
    body = "\n".join(body_lines)
    return [stmt.strip() for stmt in body.split(";\n") if stmt.strip()]


def _classify_statement(stmt: str) -> _Statement:
    """Classify one statement by its leading keyword.

    :param stmt: One statement body (no trailing ``;``).
    :returns: The classified :class:`_Statement`.
    :raises SchemaParseError: If the statement's kind is unrecognized, or an
        index statement's target table cannot be resolved.
    """
    if _CREATE_TABLE_RE.match(stmt):
        match = _TABLE_NAME_RE.search(stmt)
        if not match:
            msg = f"cannot parse table name from CREATE TABLE statement: {stmt[:80]!r}"
            raise SchemaParseError(msg)
        return _Statement("table", match.group(1), stmt)
    if _CREATE_INDEX_RE.match(stmt):
        match = _INDEX_TABLE_RE.search(stmt)
        if not match:
            msg = f"cannot determine target table for index statement: {stmt[:80]!r}"
            raise SchemaParseError(msg)
        return _Statement("index", match.group(1), stmt)
    if _CREATE_VIEW_RE.match(stmt):
        return _Statement("view", None, stmt)
    if _CREATE_TRIGGER_RE.match(stmt):
        return _Statement("trigger", None, stmt)
    msg = f"unclassifiable schema statement: {stmt[:80]!r}"
    raise SchemaParseError(msg)


def _read_schema_statements(schema_sql_path: Path) -> list[_Statement]:
    """Read and classify every statement in ``schema_sql_path``, in file order.

    :param schema_sql_path: Path to ``schema.sql``.
    :raises BuildEnvironmentError: If the file doesn't exist.
    :raises SchemaParseError: Via :func:`_classify_statement`.
    """
    if not schema_sql_path.is_file():
        msg = f"schema.sql not found: {schema_sql_path}"
        raise BuildEnvironmentError(msg)
    text = schema_sql_path.read_text()
    return [_classify_statement(stmt) for stmt in _split_statements(text)]


def _load_manifest_entries(manifest_path: Path) -> list[dict[str, object]]:
    """Read the manifest's ``artifacts`` list.

    :param manifest_path: Path to the manifest (``data-artifacts.yaml``
        shape).
    :raises BuildEnvironmentError: If the file doesn't exist.
    """
    import yaml

    if not manifest_path.is_file():
        msg = f"manifest not found: {manifest_path}"
        raise BuildEnvironmentError(msg)
    manifest = yaml.safe_load(manifest_path.read_text())
    entries: list[dict[str, object]] = manifest["artifacts"]
    return entries


def _insert_rows(
    conn: sqlite3.Connection, table: str, columns: list[str], rows: list[tuple[object, ...]]
) -> int:
    """``executemany`` a batch of already-typed row tuples into ``table``.

    :returns: Number of rows inserted (``0`` for an empty batch — no-op).
    """
    if not rows:
        return 0
    col_list = ", ".join(f'"{c}"' for c in columns)
    placeholders = ", ".join("?" for _ in columns)
    conn.executemany(f'INSERT INTO "{table}" ({col_list}) VALUES ({placeholders})', rows)
    return len(rows)


def _load_parquet_entry(conn: sqlite3.Connection, table: str, path: Path) -> int:
    """Stream ``path`` (a parquet artifact) into ``table`` batch by batch.

    Boolean/date columns need no special handling here: a parquet ``bool``
    array's ``to_pylist()`` yields Python ``bool`` (sqlite3 adapts it as an
    int automatically, since ``bool`` subclasses ``int``); a ``string``-typed
    DATE/DATETIME column yields plain ``str``/``None``, which inserts as TEXT
    matching the original source storage class.
    """
    total = 0
    parquet_file = pq.ParquetFile(path)
    for batch in parquet_file.iter_batches(65536):
        columns = list(batch.schema.names)
        column_values = [batch.column(i).to_pylist() for i in range(batch.num_columns)]
        rows = list(zip(*column_values, strict=True)) if column_values else []
        total += _insert_rows(conn, table, columns, rows)
    return total


def _coerce_csv_cell(value: str, decltype: str, column: str) -> object:
    """Reverse the exporter's CSV handling for one cell.

    The exporter (``make_data_artifacts._write_csv``) writes every value
    through the stdlib ``csv`` module with no type coercion — ``None`` and
    ``""`` are therefore indistinguishable in the file for ANY column (the
    ``csv`` module serializes both as an empty field). The builder must pick
    one interpretation per column, keyed on its declared type:

    - TEXT/VARCHAR columns: an empty field is always the string ``""`` — a
      real empty-string value round-trips correctly; a real ``NULL`` does
      NOT (it becomes ``""`` instead — a documented, accepted lossy edge of
      the CSV tier).
    - INTEGER/BIGINT/BOOLEAN/NUMERIC/FLOAT/REAL/DATE/DATETIME: an empty field
      is ``NULL`` (a numeric/date parse can't otherwise represent ``""``);
      non-empty values convert per type.

    :param value: Raw CSV cell text.
    :param decltype: The column's SQLite declared type (``PRAGMA
        table_info`` ``type``), matching
        :data:`make_data_artifacts._DECLTYPE_TO_ARROW`'s prefixes.
    :param column: The column's name — used only to name the offending
        column in :class:`SchemaParseError`'s message.
    :returns: The value to bind into the INSERT statement.
    :raises SchemaParseError: If ``decltype`` doesn't match any recognized
        prefix (TEXT/VARCHAR, INTEGER/BIGINT, BOOLEAN, NUMERIC/FLOAT/REAL,
        DATE/DATETIME) — a genuinely unrecognized declared type is a
        schema/exporter mismatch and must fail loud, never silently pass
        through as a raw string (Constitution III.11).
    """
    upper = decltype.upper()
    if upper.startswith(_TEXT_LIKE_DECLTYPE_PREFIXES):
        return value
    if value == "":
        return None
    if upper.startswith(("INTEGER", "BIGINT")):
        return int(value)
    if upper.startswith("BOOLEAN"):
        return bool(int(value))
    if upper.startswith(("NUMERIC", "FLOAT", "REAL")):
        return float(value)
    if upper.startswith(("DATE", "DATETIME")):
        return value  # source storage is already a plain string
    msg = f"column {column!r}: unrecognized CSV-tier declared type {decltype!r}"
    raise SchemaParseError(msg)


def _load_csv_entry(conn: sqlite3.Connection, table: str, path: Path) -> int:
    """Stream ``path`` (a CSV artifact) into ``table``, coercing each cell
    per its just-created column's declared type (``PRAGMA table_info``)."""
    coltypes = {row[1]: row[2] for row in conn.execute(f'PRAGMA table_info("{table}")').fetchall()}
    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.reader(handle)
        header = next(reader)
        rows = [
            tuple(
                _coerce_csv_cell(cell, coltypes[col], col)
                for cell, col in zip(raw, header, strict=True)
            )
            for raw in reader
        ]
    return _insert_rows(conn, table, header, rows)


def _load_entry(conn: sqlite3.Connection, entry: dict[str, object], sources_root: Path) -> int:
    """Verify one manifest entry's source hash, then load its rows.

    :raises BuildEnvironmentError: If the source file is missing.
    :raises SourceHashError: If the source file's sha256 doesn't match the
        manifest.
    """
    home = str(entry["home"])
    path = sources_root / home
    try:
        actual_hash = _sha256(path)
    except FileNotFoundError as error:
        msg = f"{entry['name']}: source file missing: {path}"
        raise BuildEnvironmentError(msg) from error
    expected_hash = entry["sha256"]
    if actual_hash != expected_hash:
        msg = (
            f"{entry['name']}: source hash mismatch "
            f"(manifest {expected_hash!r}, actual {actual_hash!r}, file {path})"
        )
        raise SourceHashError(msg)
    table = str(entry["source_table"])
    if entry["format"] == "csv":
        return _load_csv_entry(conn, table, path)
    return _load_parquet_entry(conn, table, path)


def build_reference_db(
    manifest_path: Path, schema_sql_path: Path, sources_root: Path, out_path: Path
) -> BuildResult:
    """Build the deterministic reference DB from a manifest + schema.sql +
    per-table sources — see the module docstring for the full algorithm.

    :param manifest_path: Manifest (``data-artifacts.yaml`` v2 shape).
    :param schema_sql_path: Canonical DDL (Task 4's extractor output).
    :param sources_root: Base directory each manifest entry's ``home`` is
        relative to (repo root for the real pipeline; a temp dir in tests).
    :param out_path: Destination SQLite file (replaced if it already exists).
    :returns: The finished build's :class:`BuildResult`.
    :raises BuildEnvironmentError: Unpinned SQLite runtime, or a required
        input file is missing.
    :raises SchemaParseError: An unclassifiable/unresolvable schema.sql
        statement.
    :raises CoverageHoleError: A schema table has no manifest source.
    :raises SourceHashError: A source file's content drifted from its
        manifest hash.

    .. note::
       If a :class:`SourceHashError` (or any error raised once the output
       file has been created and DDL applied — during per-table loading)
       fires mid-build, ``out_path`` is left on disk as a PARTIAL, incomplete
       database. That's not silently mistaken for a good build: the next
       call to :func:`build_reference_db` targeting the same ``out_path``
       unlinks it first (step 2) before writing again. Failures raised
       BEFORE the output file is created — unpinned SQLite runtime, a
       missing manifest/schema.sql, or a coverage hole (all detected in the
       pre-flight checks above) — leave whatever was already at ``out_path``
       untouched: nothing, for a fresh path (the common case).
    """
    if sqlite3.sqlite_version != PINNED_SQLITE_VERSION:
        msg = f"runtime sqlite3 {sqlite3.sqlite_version} != pinned {PINNED_SQLITE_VERSION}"
        raise BuildEnvironmentError(msg)

    entries = _load_manifest_entries(manifest_path)
    statements = _read_schema_statements(schema_sql_path)

    table_statements = [s for s in statements if s.kind == "table"]
    table_names = {s.table for s in table_statements if s.table is not None}
    index_statements = [s for s in statements if s.kind == "index"]
    trailing_statements = [s for s in statements if s.kind in ("view", "trigger")]

    # Replay contract: the builder executes tables, then data, then indexes,
    # then views/triggers — so it can only reproduce schema.sql's rowid order
    # byte-for-byte when the file already has that contiguous-block shape
    # (the real DB's is exactly TABLE* INDEX* VIEW*). An interleaved file
    # would be silently reordered in the product; fail loud instead
    # (Constitution III.11).
    kind_rank = {"table": 0, "index": 1, "view": 2, "trigger": 2}
    ranks = [kind_rank[s.kind] for s in statements]
    if ranks != sorted(ranks):
        msg = (
            "schema.sql interleaves statement kinds (expected all tables, "
            "then all indexes, then views/triggers) — the builder's replay "
            "order cannot reproduce its sqlite_master rowid order"
        )
        raise SchemaParseError(msg)

    entry_source_tables = {str(entry["source_table"]) for entry in entries}
    missing_sources = sorted(name for name in table_names if name not in entry_source_tables)
    if missing_sources:
        msg = f"schema table(s) with no manifest source: {missing_sources}"
        raise CoverageHoleError(msg)

    if out_path.exists():
        out_path.unlink()
    out_path.parent.mkdir(parents=True, exist_ok=True)

    conn = sqlite3.connect(out_path)
    try:
        conn.execute(f"PRAGMA page_size={PAGE_SIZE}")
        conn.execute("PRAGMA journal_mode=DELETE")
        # VACUUM spills a full copy of the DB to sqlite's temp dir, which
        # defaults to /tmp — a 16G tmpfs on the dev box, where a ~4.6GB
        # spill dies "database or disk is full" (cutover Step 4, 2026-07-20).
        # Pin temp next to the output file instead: its filesystem must hold
        # the product anyway. (Deprecated pragma, but it is the only
        # per-connection control; SQLITE_TMPDIR proved unreliable through
        # the mise->poetry->python chain.)
        conn.execute(f"PRAGMA temp_store_directory = '{out_path.parent.resolve()}'")

        for stmt in table_statements:
            conn.execute(stmt.sql)

        rows_inserted = 0
        tables_built = 0
        for entry in entries:
            table = str(entry["source_table"])
            if table not in table_names:
                # e.g. a register-mode entry whose table was already demoted
                # out of the schema — the membership test's other half.
                continue
            rows_inserted += _load_entry(conn, entry, sources_root)
            tables_built += 1

        # All indexes AFTER all loads (bulk-load speed), in schema.sql
        # statement order — sqlite_master rowid order is part of the schema
        # fixed-point contract, and per-table creation in manifest (name-
        # sorted) order broke it against the real DB's historical order
        # (cutover Step 3, 2026-07-20).
        for stmt in index_statements:
            conn.execute(stmt.sql)

        for stmt in trailing_statements:
            conn.execute(stmt.sql)

        conn.execute(f"PRAGMA application_id={APPLICATION_ID}")
        conn.execute(f"PRAGMA user_version={USER_VERSION}")
        conn.commit()
        conn.execute("VACUUM")
    finally:
        conn.close()

    return BuildResult(
        path=out_path,
        sha256=_sha256(out_path),
        sqlite_version=sqlite3.sqlite_version,
        tables_built=tables_built,
        rows_inserted=rows_inserted,
    )


def main(argv: list[str] | None = None) -> int:
    """CLI: build the reference DB, optionally stamping the manifest's
    ``product`` block with this build's identity (``--update-manifest``)."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--schema", type=Path, default=DEFAULT_SCHEMA)
    parser.add_argument("--sources-root", type=Path, default=DEFAULT_SOURCES_ROOT)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument(
        "--update-manifest",
        action="store_true",
        help="stamp the manifest's product block with this build's sha256/pragmas",
    )
    args = parser.parse_args(argv)

    result = build_reference_db(args.manifest, args.schema, args.sources_root, args.out)

    if args.update_manifest:
        update_product_block(
            args.manifest,
            {
                "name": result.path.name,
                "sha256": result.sha256,
                "page_size": PAGE_SIZE,
                "application_id": APPLICATION_ID,
                "user_version": USER_VERSION,
                "sqlite_version": result.sqlite_version,
            },
        )

    print(
        f"[build] {result.path}: {result.tables_built} tables, "
        f"{result.rows_inserted} rows, sha256={result.sha256}"
    )
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except ReferenceBuildError as error:
        print(f"[build] ABORT: {error}", file=sys.stderr)
        sys.exit(2)
