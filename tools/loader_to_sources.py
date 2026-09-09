#!/usr/bin/env python3
"""Loader-to-sources wrapper for legacy DB-writing loaders (parquet-canonical
cutover plan, Task 11).

Before the parquet-canonical cutover, a loader like ``ingest_bea_imports.py``
opened the shared reference DB directly and wrote rows into it. Post-cutover,
the DB is a build PRODUCT (``dist/build/marxist-data-3NF.sqlite``, rebuilt
from the manifest + ``schema.sql`` + per-table parquet/CSV sources by
``tools/build_reference_db.py``) — nothing is allowed to write it directly,
loaders included.

This wrapper is the seam: it runs a legacy loader's ``main(argv) -> int``
against a throwaway SCRATCH COPY of the build product, then re-exports only
the tables that loader is declared to affect as parquet sources (via
``tools.make_data_artifacts.export_table_parquet``, the one parquet-writing
path every other exporter uses) and regenerates the manifest (via
``tools.make_data_artifacts._rewrite_manifest_preserving_blocks``, the Task 3
writer that preserves any existing ``schema``/``product`` block). The scratch
copy is always deleted; the shared build product and the working DB are never
opened for write by this module.

**The invariant:** loaders produce SOURCES; only the builder
(``tools/build_reference_db.py``) produces the DB.

Usage::

    uv run python tools/loader_to_sources.py \\
        --loader ingest_bea_imports \\
        --tables fact_bea_io_coefficient,dim_bea_io_table_type,dim_time
"""

from __future__ import annotations

import argparse
import importlib
import shutil
import sqlite3
import sys
import tempfile
from collections.abc import Callable
from pathlib import Path
from typing import cast

sys.path.insert(0, str(Path(__file__).resolve().parent))
import make_data_artifacts as mda  # noqa: E402

#: Default canonical locations (repo-root-relative), matching the builder's
#: ``data:build-db`` mise task (``--out dist/build/marxist-data-3NF.sqlite``)
#: and ``build_reference_db.py``'s ``DEFAULT_SOURCES_ROOT`` precedent.
DEFAULT_BUILD_PRODUCT = Path("dist/build/marxist-data-3NF.sqlite")
DEFAULT_SOURCES_ROOT = Path()


class LoaderToSourcesError(Exception):
    """A loader-to-sources run failed loudly — bad input, a loader that
    reported failure, or a table this wrapper cannot re-export."""


def run_loader_to_sources(
    loader_main: Callable[[list[str]], int],
    affected_tables: list[str],
    build_product: Path,
    sources_root: Path,
) -> list[str]:
    """Run a legacy DB-writing loader against a SCRATCH COPY of the build product,
    then re-export the affected tables as parquet sources + regenerate the manifest.

    The invariant this enforces: loaders produce SOURCES; only the builder produces
    the DB. The scratch copy is deleted; the shared DB is never opened for write.
    Returns the affected tables whose content actually changed.

    :param loader_main: The legacy loader's ``main(argv) -> int`` entry point
        (e.g. ``ingest_bea_imports.main``). Called exactly once, as
        ``loader_main(["--db-url", f"sqlite:///{scratch}"])``.
    :param affected_tables: Source-table names (``data-artifacts.yaml``
        ``source_table`` values) the loader is declared to write. Each must
        already have a ``format: parquet`` manifest entry — this wrapper only
        re-exports via :func:`make_data_artifacts.export_table_parquet`.
    :param build_product: The current build product to copy before handing
        the scratch copy to ``loader_main`` (never opened for write itself).
    :param sources_root: Base directory each manifest entry's ``home`` is
        relative to (repo root in production; a temp dir in tests) — mirrors
        ``build_reference_db.build_reference_db``'s ``sources_root`` contract.
    :returns: The subset of ``affected_tables`` whose re-exported source
        bytes differ from what was on disk before this run.
    :raises LoaderToSourcesError: ``build_product`` is missing, the manifest
        is missing, an affected table has no manifest entry or a non-parquet
        one, or ``loader_main`` returns a non-zero exit code.
    """
    if not build_product.is_file():
        msg = f"build product not found: {build_product}"
        raise LoaderToSourcesError(msg)

    manifest_path = mda.MANIFEST_PATH
    manifest = mda._read_manifest(manifest_path)
    if manifest is None:
        msg = f"manifest missing: {manifest_path}"
        raise LoaderToSourcesError(msg)
    entries = cast("list[dict[str, object]]", manifest["artifacts"])
    entry_by_table = {str(entry["source_table"]): entry for entry in entries}

    missing = [t for t in affected_tables if t not in entry_by_table]
    if missing:
        msg = f"affected table(s) have no manifest entry: {missing}"
        raise LoaderToSourcesError(msg)
    non_parquet = [t for t in affected_tables if entry_by_table[t]["format"] != "parquet"]
    if non_parquet:
        msg = f"affected table(s) are not parquet-format manifest artifacts: {non_parquet}"
        raise LoaderToSourcesError(msg)

    changed: list[str] = []
    with tempfile.NamedTemporaryFile(suffix=".sqlite") as scratch_handle:
        scratch_path = Path(scratch_handle.name)
        shutil.copyfile(build_product, scratch_path)

        exit_code = loader_main(["--db-url", f"sqlite:///{scratch_path}"])
        if exit_code != 0:
            msg = f"loader_main exited {exit_code} against the scratch copy {scratch_path}"
            raise LoaderToSourcesError(msg)

        conn = sqlite3.connect(f"file:{scratch_path}?mode=ro", uri=True)
        try:
            for table in affected_tables:
                entry = entry_by_table[table]
                dest = sources_root / str(entry["home"])
                before_hash = mda._sha256(dest) if dest.is_file() else None
                rows, _size = mda.export_table_parquet(conn, table, dest)
                after_hash = mda._sha256(dest)
                entry["rows"] = rows
                entry["sha256"] = after_hash
                if after_hash != before_hash:
                    changed.append(table)
        finally:
            conn.close()
        # `with` block exit deletes scratch_path — no manual cleanup needed.

    mda._rewrite_manifest_preserving_blocks(entries, manifest_path=manifest_path)
    return changed


def _resolve_loader_main(module_name: str) -> Callable[[list[str]], int]:
    """Import ``module_name`` (resolved on ``tools/``'s own sys.path entry,
    inserted above) and return its ``main(argv) -> int`` callable.

    :raises LoaderToSourcesError: If the module has no callable ``main``.
    """
    module = importlib.import_module(module_name)
    loader_main = getattr(module, "main", None)
    if loader_main is None or not callable(loader_main):
        msg = f"loader module {module_name!r} has no callable main(argv) -> int"
        raise LoaderToSourcesError(msg)
    return cast("Callable[[list[str]], int]", loader_main)


def main(argv: list[str] | None = None) -> int:
    """CLI: run ``--loader``'s ``main`` against a scratch copy of
    ``--build-product``, re-export ``--tables`` as parquet sources, and
    regenerate the manifest."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--loader",
        required=True,
        help="tools/ module name exposing main(argv) -> int (e.g. ingest_bea_imports)",
    )
    parser.add_argument(
        "--tables", required=True, help="comma-separated affected source_table names"
    )
    parser.add_argument("--build-product", type=Path, default=DEFAULT_BUILD_PRODUCT)
    parser.add_argument("--sources-root", type=Path, default=DEFAULT_SOURCES_ROOT)
    args = parser.parse_args(argv)

    loader_main = _resolve_loader_main(args.loader)
    affected_tables = [t.strip() for t in args.tables.split(",") if t.strip()]
    if not affected_tables:
        msg = "--tables produced an empty affected-table list"
        raise LoaderToSourcesError(msg)

    changed = run_loader_to_sources(
        loader_main, affected_tables, args.build_product, args.sources_root
    )

    print(f"[loader-to-sources] loader={args.loader} affected={affected_tables}")
    if changed:
        print(f"[loader-to-sources] content changed: {changed}")
    else:
        print("[loader-to-sources] no affected table's content changed")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except LoaderToSourcesError as error:
        print(f"[loader-to-sources] ABORT: {error}", file=sys.stderr)
        sys.exit(2)
