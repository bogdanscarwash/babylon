#!/usr/bin/env python3
"""Content round-trip verifier for the parquet-canonical reference pipeline
(plan 2026-07-19, Task 7) — the export-verify gate the cutover depends on.

Compares a "live" reference DB against a "rebuilt" build product
(``tools/build_reference_db.py``'s output) table-by-table and view-by-view via
a canonicalized content hash, so the pipeline's documented, accepted
normalizations never register as drift while any real value change does
(Constitution III.11 — loud failure, never a silent default).

Value canonicalization (the D2/NUMERIC rule, stated once, tested):

- Columns whose declared type maps to ``pa.float64()`` in
  ``make_data_artifacts._DECLTYPE_TO_ARROW`` (NUMERIC/FLOAT/REAL) hash
  ``quote(CAST(col AS REAL))`` — this makes SQLite's own NUMERIC-affinity
  storage-class drift (INTEGER 1 vs REAL 1.0, the same logical value) invisible
  to the comparison, exactly like the exporter's declared NUMERIC->float64
  normalization does for parquet.
- Columns whose declared type maps to ``pa.bool_()`` (BOOLEAN) hash
  ``quote(CASE WHEN col IS NULL THEN NULL WHEN col THEN 1 ELSE 0 END)`` —
  mirroring ``make_data_artifacts._column_array``'s export-time coercion
  (``bool(v)`` collapses any non-0/1 int, e.g. ``2``, to ``True``/``1``), so a
  rebuilt DB storing ``1`` where the live DB stored ``2`` compares equal.
- Every other column hashes plain ``quote(col)``.
- Views carry no decltypes, so they always hash plain ``quote()`` over every
  column, ordered by every column.
- ``sqlite_%`` internal tables/objects are excluded (build-history, not data).

Both DBs are opened ``mode=ro`` (URI) — this tool is incapable of writing to
either. Hashing streams row-by-row over a server-side cursor (never
``fetchall()``) — the real reference DB has multi-million-row tables.

Usage::

    uv run python tools/verify_reference_roundtrip.py \\
        --live data/sqlite/marxist-data-3NF.sqlite \\
        --rebuilt dist/build/marxist-data-3NF.sqlite
"""

from __future__ import annotations

import argparse
import dataclasses
import hashlib
import sqlite3
import sys
from collections.abc import Callable
from pathlib import Path

import pyarrow as pa  # type: ignore[import-untyped]

sys.path.insert(0, str(Path(__file__).resolve().parent))
from make_data_artifacts import (  # noqa: E402
    _arrow_type,
    _column_info,
    _table_layout_or_all_columns,
    governed_db_tables,
)

#: Row/column separators for the streaming hash. ``quote()`` never emits these
#: raw control bytes itself (TEXT is escaped via doubled ``'``, BLOB via hex
#: ``X'...'``) — the only way they could appear is a TEXT value literally
#: containing them, a non-issue for this reference DB's economic/geographic
#: content.
_COLUMN_SEP = "\x1f"
_ROW_SEP = b"\x1e"


class RoundtripError(Exception):
    """Base for verifier failures — callers only catch this or a specific
    subclass (Constitution III.11: no generic catch-alls)."""


class DatabaseNotFoundError(RoundtripError):
    """A ``--live``/``--rebuilt`` database path doesn't exist."""


@dataclasses.dataclass(frozen=True)
class RoundtripReport:
    """The full :func:`compare_databases` verdict.

    :ivar ok: ``True`` iff every table and view verdict is ``"ok"``.
    :ivar tables: per-table verdict, keyed by table name: ``"ok"``,
        ``"row-count: L vs R"``, ``"content-hash mismatch"``,
        ``"missing-in-live"``, or ``"missing-in-rebuilt"``.
    :ivar views: the same verdict shape, keyed by view name.
    """

    ok: bool
    tables: dict[str, str]
    views: dict[str, str]


def _canonical_column_expr(column: str, decltype: str) -> str:
    """The ``quote(...)`` SQL expression for one table column, canonicalized
    per its declared type — see the module docstring's D2/BOOLEAN rules.

    :param column: Column name (from ``PRAGMA table_info``).
    :param decltype: The column's declared type (same source).
    :raises ArtifactError: Via :func:`make_data_artifacts._arrow_type`, if
        ``decltype`` has no pinned arrow mapping.
    """
    quoted_col = f'"{column}"'
    arrow_type = _arrow_type(decltype)
    if pa.types.is_boolean(arrow_type):
        return (
            f"quote(CASE WHEN {quoted_col} IS NULL THEN NULL WHEN {quoted_col} THEN 1 ELSE 0 END)"
        )
    if pa.types.is_floating(arrow_type):
        return f"quote(CAST({quoted_col} AS REAL))"
    return f"quote({quoted_col})"


def _hash_rows(cursor: sqlite3.Cursor) -> tuple[int, str]:
    """Stream ``cursor`` row-by-row into a sha256 digest — never materializes
    the full result set (the real reference DB has multi-million-row tables).

    :param cursor: An open cursor over a ``SELECT`` whose every column is
        already a canonicalized ``quote(...)`` expression.
    :returns: ``(row count, hex digest)``.
    """
    digest = hashlib.sha256()
    count = 0
    for row in cursor:
        digest.update(_COLUMN_SEP.join(str(value) for value in row).encode("utf-8"))
        digest.update(_ROW_SEP)
        count += 1
    return count, digest.hexdigest()


def table_content_hash(conn: sqlite3.Connection, table: str) -> tuple[int, str]:
    """Row count + sha256 over ``table``'s canonicalized, ordered rows.

    Ordering mirrors the exporter's own determinism pin: primary-key order, or
    every column for a PK-less table (:func:`make_data_artifacts.
    _table_layout_or_all_columns`'s fallback, imported rather than
    reimplemented — a PK-less table's identical rows are interchangeable, so
    sorting by every column stays deterministic without a declared key).

    :param conn: Open connection to the DB (live or rebuilt).
    :param table: Table name.
    :returns: ``(row count, hex digest)``.
    """
    columns, pk, _schema = _table_layout_or_all_columns(conn, table)
    decltypes = {row[1]: row[2] for row in _column_info(conn, table)}
    select_cols = ", ".join(_canonical_column_expr(c, decltypes[c]) for c in columns)
    order = ", ".join(f'"{c}"' for c in pk)
    cursor = conn.execute(
        f'SELECT {select_cols} FROM "{table}" ORDER BY {order}'  # noqa: S608 — table/columns from PRAGMA, not user input
    )
    return _hash_rows(cursor)


def view_content_hash(conn: sqlite3.Connection, view: str) -> tuple[int, str]:
    """Row count + sha256 over ``view``'s rows — plain ``quote()`` on every
    column (views carry no decltypes, so no NUMERIC/BOOLEAN canonicalization
    applies), ordered by every column.

    :param conn: Open connection to the DB (live or rebuilt).
    :param view: View name.
    :returns: ``(row count, hex digest)``.
    """
    columns = [row[1] for row in conn.execute(f'PRAGMA table_info("{view}")').fetchall()]
    select_cols = ", ".join(f'quote("{c}")' for c in columns)
    order = ", ".join(f'"{c}"' for c in columns)
    cursor = conn.execute(
        f'SELECT {select_cols} FROM "{view}" ORDER BY {order}'  # noqa: S608 — view/columns from PRAGMA, not user input
    )
    return _hash_rows(cursor)


def _governed_db_views(conn: sqlite3.Connection) -> list[str]:
    """Every governed view in ``conn`` — the view-side twin of
    :func:`make_data_artifacts.governed_db_tables`, scoped by the same
    ``GOVERNED_PREFIXES`` boundary (all reference views are ``view_*``)."""
    from babylon.data.catalog import GOVERNED_PREFIXES

    rows = conn.execute(
        "SELECT name FROM sqlite_master "
        "WHERE type = 'view' AND name NOT LIKE 'sqlite_%' "
        "ORDER BY name"
    ).fetchall()
    return [row[0] for row in rows if row[0].startswith(GOVERNED_PREFIXES)]


def _compare_names(
    live_conn: sqlite3.Connection,
    rebuilt_conn: sqlite3.Connection,
    live_names: list[str],
    rebuilt_names: list[str],
    hash_fn: Callable[[sqlite3.Connection, str], tuple[int, str]],
) -> dict[str, str]:
    """Shared verdict logic for one kind of object (table or view).

    :param live_conn: Open connection to the live DB.
    :param rebuilt_conn: Open connection to the rebuilt DB.
    :param live_names: Object names present in the live DB.
    :param rebuilt_names: Object names present in the rebuilt DB.
    :param hash_fn: :func:`table_content_hash` or :func:`view_content_hash`.
    :returns: Verdict per name, sorted by name.
    """
    live_set = set(live_names)
    rebuilt_set = set(rebuilt_names)
    verdicts: dict[str, str] = {}
    for name in sorted(live_set | rebuilt_set):
        if name not in live_set:
            verdicts[name] = "missing-in-live"
            continue
        if name not in rebuilt_set:
            verdicts[name] = "missing-in-rebuilt"
            continue
        live_count, live_hash = hash_fn(live_conn, name)
        rebuilt_count, rebuilt_hash = hash_fn(rebuilt_conn, name)
        if live_count != rebuilt_count:
            verdicts[name] = f"row-count: {live_count} vs {rebuilt_count}"
        elif live_hash != rebuilt_hash:
            verdicts[name] = "content-hash mismatch"
        else:
            verdicts[name] = "ok"
    return verdicts


def compare_databases(live: Path, rebuilt: Path) -> RoundtripReport:
    """Content-compare every table and view between ``live`` and ``rebuilt``.

    Opens both DBs ``mode=ro`` (URI) — this function is incapable of writing
    to either.

    :param live: Path to the live reference DB.
    :param rebuilt: Path to the rebuilt build product.
    :returns: The full :class:`RoundtripReport`.
    :raises DatabaseNotFoundError: If either path doesn't exist.
    """
    if not live.exists():
        msg = f"live database not found: {live}"
        raise DatabaseNotFoundError(msg)
    if not rebuilt.exists():
        msg = f"rebuilt database not found: {rebuilt}"
        raise DatabaseNotFoundError(msg)

    live_conn = sqlite3.connect(f"file:{live}?mode=ro", uri=True)
    rebuilt_conn = sqlite3.connect(f"file:{rebuilt}?mode=ro", uri=True)
    try:
        tables = _compare_names(
            live_conn,
            rebuilt_conn,
            governed_db_tables(live_conn),
            governed_db_tables(rebuilt_conn),
            table_content_hash,
        )
        views = _compare_names(
            live_conn,
            rebuilt_conn,
            _governed_db_views(live_conn),
            _governed_db_views(rebuilt_conn),
            view_content_hash,
        )
    finally:
        live_conn.close()
        rebuilt_conn.close()

    ok = all(v == "ok" for v in tables.values()) and all(v == "ok" for v in views.values())
    return RoundtripReport(ok=ok, tables=tables, views=views)


def main(argv: list[str] | None = None) -> int:
    """CLI: print every non-``ok`` table/view verdict, exit 1 on any mismatch
    (Constitution III.11 — loud failure)."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--live", type=Path, required=True, help="path to the live reference DB")
    parser.add_argument(
        "--rebuilt", type=Path, required=True, help="path to the rebuilt build product"
    )
    args = parser.parse_args(argv)

    report = compare_databases(args.live, args.rebuilt)

    for kind, verdicts in (("table", report.tables), ("view", report.views)):
        for name, verdict in sorted(verdicts.items()):
            if verdict != "ok":
                print(f"[roundtrip] {kind} {name}: {verdict}", file=sys.stderr)

    total = len(report.tables) + len(report.views)
    print(
        f"[roundtrip] {total} objects checked "
        f"({len(report.tables)} tables, {len(report.views)} views), ok={report.ok}"
    )
    return 0 if report.ok else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except RoundtripError as error:
        print(f"[roundtrip] ABORT: {error}", file=sys.stderr)
        sys.exit(2)
