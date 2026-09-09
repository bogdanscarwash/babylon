"""Behavioral contract for ``tools/loader_to_sources.py`` (parquet-canonical
cutover plan, Task 11).

The invariant under test: a legacy DB-writing loader (``ingest_bea_imports.main``
and friends) must never open the shared build product for write. The wrapper
runs the loader against a throwaway scratch copy, re-exports only the affected
tables as parquet sources, regenerates the manifest, and reports which of
those tables' content actually changed — everything else in the manifest
(and the original build product on disk) must come out byte-identical.

Synthetic (not real-BEA) input throughout: a toy loader inserting one row
into a scratch sqlite DB, proven against a two-table manifest fixture so the
"all other entries' hashes unchanged" law has something to violate if the
wrapper regenerated more than it was told to.
"""

from __future__ import annotations

import argparse
import sqlite3
import sys
import tempfile
from pathlib import Path

import pytest
import yaml

_REPO_ROOT = Path(__file__).resolve().parents[3]
TOOLS_DIR = _REPO_ROOT / "tools"
sys.path.insert(0, str(TOOLS_DIR))

import make_data_artifacts  # type: ignore[import-not-found]  # noqa: E402
from loader_to_sources import (  # type: ignore[import-not-found]  # noqa: E402
    LoaderToSourcesError,
    run_loader_to_sources,
)
from make_data_artifacts import (  # type: ignore[import-not-found]  # noqa: E402
    _sha256,
    _write_manifest,
    export_table_parquet,
)


def _seed_build_product(path: Path) -> None:
    """A toy two-table build product: ``fact_sample`` (the loader's target)
    and ``dim_untouched`` (proves the wrapper never touches un-affected
    tables' sources)."""
    conn = sqlite3.connect(path)
    conn.executescript(
        """
        CREATE TABLE fact_sample (
            id INTEGER NOT NULL,
            label VARCHAR(20),
            value NUMERIC(10, 2),
            PRIMARY KEY (id)
        );
        INSERT INTO fact_sample VALUES (1, 'a', 1.25), (2, 'b', 2.5);
        CREATE TABLE dim_untouched (
            id INTEGER NOT NULL,
            label VARCHAR(20),
            PRIMARY KEY (id)
        );
        INSERT INTO dim_untouched VALUES (1, 'x'), (2, 'y');
        """
    )
    conn.commit()
    conn.close()


def _toy_loader_main(argv: list[str]) -> int:
    """Synthetic legacy loader: parses ``--db-url`` (mirroring
    ``ingest_bea_imports.main``'s real contract) and inserts one new row
    into ``fact_sample`` via a plain sqlite3 connection."""
    parser = argparse.ArgumentParser()
    parser.add_argument("--db-url", required=True)
    args = parser.parse_args(argv)
    db_path = args.db_url.removeprefix("sqlite:///")
    conn = sqlite3.connect(db_path)
    try:
        conn.execute("INSERT INTO fact_sample (id, label, value) VALUES (99, 'new', 9.9)")
        conn.commit()
    finally:
        conn.close()
    return 0


def _noop_loader_main(argv: list[str]) -> int:
    """A loader that touches nothing — proves the wrapper reports NO change
    (not a spurious one) when the underlying content is identical."""
    parser = argparse.ArgumentParser()
    parser.add_argument("--db-url", required=True)
    parser.parse_args(argv)
    return 0


def _failing_loader_main(argv: list[str]) -> int:
    return 1


@pytest.fixture
def build_product(tmp_path: Path) -> Path:
    path = tmp_path / "build" / "marxist-data-3NF.sqlite"
    path.parent.mkdir(parents=True)
    _seed_build_product(path)
    return path


@pytest.fixture
def sources_root(tmp_path: Path) -> Path:
    return tmp_path / "repo"


@pytest.fixture
def manifest_path(
    tmp_path: Path,
    build_product: Path,
    sources_root: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> Path:
    """A two-entry manifest, generated from the SAME build product, pointing
    at real parquet sources under ``sources_root`` — the pre-condition the
    wrapper is handed in production (manifest and sources in sync)."""
    conn = sqlite3.connect(f"file:{build_product}?mode=ro", uri=True)
    entries: list[dict[str, object]] = []
    try:
        for table in ("fact_sample", "dim_untouched"):
            home = f"dist/data-artifacts/{table}.parquet"
            dest = sources_root / home
            rows, _size = export_table_parquet(conn, table, dest)
            entries.append(
                {
                    "name": table,
                    "format": "parquet",
                    "source_table": table,
                    "mode": "generate",
                    "rows": rows,
                    "sha256": _sha256(dest),
                    "home": home,
                    "material_relation": f"test fixture artifact for {table}.",
                }
            )
    finally:
        conn.close()
    manifest = tmp_path / "data-artifacts.yaml"
    _write_manifest(entries, path=manifest)
    monkeypatch.setattr(make_data_artifacts, "MANIFEST_PATH", manifest)
    return manifest


@pytest.fixture
def scratch_dir(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    """An empty, dedicated tempdir — ``tempfile.tempdir`` is pointed here so
    the test can assert the scratch copy left nothing behind."""
    directory = tmp_path / "scratch"
    directory.mkdir()
    monkeypatch.setattr(tempfile, "tempdir", str(directory))
    return directory


class TestRunLoaderToSources:
    def test_toy_loader_changes_only_the_affected_source(
        self,
        build_product: Path,
        sources_root: Path,
        manifest_path: Path,
        scratch_dir: Path,
    ) -> None:
        before_product_hash = _sha256(build_product)
        before = yaml.safe_load(manifest_path.read_text())
        before_fact = next(e for e in before["artifacts"] if e["name"] == "fact_sample")
        before_untouched = next(e for e in before["artifacts"] if e["name"] == "dim_untouched")

        changed = run_loader_to_sources(
            _toy_loader_main, ["fact_sample"], build_product, sources_root
        )

        assert changed == ["fact_sample"]

        after = yaml.safe_load(manifest_path.read_text())
        after_fact = next(e for e in after["artifacts"] if e["name"] == "fact_sample")
        after_untouched = next(e for e in after["artifacts"] if e["name"] == "dim_untouched")

        # Affected entry's hash + row count moved.
        assert after_fact["rows"] == 3
        assert after_fact["sha256"] != before_fact["sha256"]

        # Every other entry is byte-for-byte untouched.
        assert after_untouched == before_untouched

        # The scratch copy is gone.
        assert list(scratch_dir.iterdir()) == []

        # The original build product was never opened for write.
        assert _sha256(build_product) == before_product_hash

        # The re-exported parquet actually carries the loader's new row.
        import pyarrow.parquet as pq

        table = pq.read_table(sources_root / "dist/data-artifacts/fact_sample.parquet")
        assert table.num_rows == 3
        assert sorted(table.column("id").to_pylist()) == [1, 2, 99]

    def test_noop_loader_reports_no_change(
        self,
        build_product: Path,
        sources_root: Path,
        manifest_path: Path,
        scratch_dir: Path,
    ) -> None:
        before = yaml.safe_load(manifest_path.read_text())

        changed = run_loader_to_sources(
            _noop_loader_main, ["fact_sample"], build_product, sources_root
        )

        assert changed == []
        after = yaml.safe_load(manifest_path.read_text())
        assert after == before
        assert list(scratch_dir.iterdir()) == []

    def test_missing_manifest_entry_is_loud(
        self,
        build_product: Path,
        sources_root: Path,
        manifest_path: Path,
        scratch_dir: Path,
    ) -> None:
        with pytest.raises(LoaderToSourcesError, match="fact_nonexistent"):
            run_loader_to_sources(
                _toy_loader_main, ["fact_nonexistent"], build_product, sources_root
            )
        assert list(scratch_dir.iterdir()) == []  # never got as far as a scratch copy

    def test_loader_failure_is_loud_and_still_cleans_up(
        self,
        build_product: Path,
        sources_root: Path,
        manifest_path: Path,
        scratch_dir: Path,
    ) -> None:
        with pytest.raises(LoaderToSourcesError, match="exited 1"):
            run_loader_to_sources(
                _failing_loader_main, ["fact_sample"], build_product, sources_root
            )
        assert list(scratch_dir.iterdir()) == []

    def test_missing_build_product_is_loud(
        self,
        tmp_path: Path,
        sources_root: Path,
        manifest_path: Path,
    ) -> None:
        missing = tmp_path / "nope.sqlite"
        with pytest.raises(LoaderToSourcesError, match="build product not found"):
            run_loader_to_sources(_toy_loader_main, ["fact_sample"], missing, sources_root)
