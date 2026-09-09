# Parquet Reference-Data Pipeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking. **Execution is owner-gated: Phases 0–5 may start only
> after the owner approves this plan; Phase 6 (cutover) additionally runs ALONE on its own branch
> with nothing else in flight.**

**Goal:** Invert reference-data authority per the approved spec
(`docs/superpowers/specs/2026-07-18-parquet-reference-pipeline-design.md`): per-table parquet
sources + `schema.sql` become canonical, and `data/sqlite/marxist-data-3NF.sqlite` becomes a
deterministic, hash-pinned build product — then land IMPORT_USE as the first source-only ingest,
unblocking the Φ/imperial-rent data path.

**Architecture:** Extend the proven ADR076 export discipline (`tools/make_data_artifacts.py`:
PK-sorted rows, explicit arrow schema, zstd-9, single row group, double-generation byte proof)
from 8 curated tables to full DB coverage; add a DDL extractor (`schema.sql`), a deterministic
builder (`tools/build_reference_db.py`), and a content round-trip verifier; rebase the CI subset
and the `fetch-reference-db` action onto the rebuilt product; finally flip authority and re-route
`tools/ingest_bea_imports.py` through a loader→sources wrapper.

**Tech Stack:** Python 3.11, sqlite3 stdlib (runtime pinned 3.46.1), pyarrow (already a dep,
pinned by `poetry.lock`), PyYAML, existing sentinel framework (`babylon.sentinels.coverage`),
mise tasks, GitHub Releases (ci-data channel) + composite action.

## Global Constraints

- **CI and tests never touch `/media/user/data`** (owner ruling 2026-07-14). All drive
  operations in this plan are dev-box-only mise/manual tasks; CI consumes release assets only.
- **The vol3 worktree (`babylon-vol3`), branch `refactor/vol3-money-scissors`, and
  `tests/integration/test_grundrisse_cycle.py` are untouchable.**
- **`mise run qa:regression` must be 5/5 byte-identical after every task in Phases 0–5** (these
  phases change no runtime data). In Phase 6, gate 3 of the spec governs; the IMPORT_USE landing
  (Task 11) is the one step where drift is plausible — if it drifts, it proceeds ONLY as a
  declared `test(baselines):` ceremony commit with a per-scenario drift table (pre-authorization
  for this specific ceremony is requested as part of this plan's approval).
- **Determinism pins (existing, reused verbatim):** parquet codec `zstd` level 9, single row
  group, PK-sorted rows, explicit arrow schema from declared types (`_DECLTYPE_TO_ARROW` — NUMERIC
  → float64 is a declared, documented normalization), pyarrow via `poetry.lock`. **New pins added
  by this plan:** `page_size=4096` (measured on the live DB), `application_id=0x4241424C`
  ("BABL"), `user_version=1`, runtime SQLite version `3.46.1` asserted at build time,
  `journal_mode=DELETE` during build + final `VACUUM` before hashing (the WAL footgun rule already
  documented in `make_reference_subset.py::_vacuum_output`). Changing ANY pin is a declared
  regeneration event.
- **`data-artifacts.yaml` is regenerated wholesale by tooling, never hand-edited** (existing
  rule; this plan keeps a single writer module and adds a builder-owned `product` block).
- **Catalog sentinel bijection** (`sentinels/coverage/db_probe.py::check_catalog_db_reconciliation`)
  must stay green at every step; catalog rows and registry entries reconcile 1:1.
- **2 GB GitHub release-asset cap:** measured in Phase 0. Any single table whose parquet exceeds
  1.8 GB (0.9 safety factor) is a loud STOP (exit 2) — the sharding contingency (§Deviations D3)
  is then specced and approved before continuing, never silently implemented.
- **Layering:** `tools/` may import `babylon.*`; nothing in `src/babylon` may import from
  `tools/` or `web/` (`mise run lint:imports`).
- Conventional commits per task, `mise run commit -- "..."`, verify HEAD moved after every
  commit, trailer `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`, never `git add -A`.
- Test runs single-flight; scoped `mise run test:q -- <path>` locally; full-DB operations
  (export/build/verify) are dev-box tasks, run one at a time.

## Deviations from the spec (flagged for owner review — approve with the plan)

- **D1 — IMPORT_USE lands as ROWS, not a new table.** Spec §3.5 says "emits
  `fact_bea_import_use.parquet` … the DB gains the table". The written loader
  (`tools/ingest_bea_imports.py`) actually INSERTs `FactBEAIOCoefficient` rows with
  `table_type='IMPORT_USE'` into the existing `fact_bea_io_coefficient` table (plus a
  `dim_bea_io_table_type` row), and the consumer
  (`babylon.domain.economics.tensor_hierarchy.production_chain_rent.DBImportShareSource`) reads it
  from there. This plan is faithful to the code: the ingest regenerates the
  `fact_bea_io_coefficient` (+ `dim_bea_io_table_type`, `dim_time`) parquet sources. No new table,
  no consumer changes.
- **D2 — container-hash pins the build product, not the working copy.** `reference/database.py`
  sets `journal_mode=WAL` on connect, so the dev box's working DB bytes change on first open by
  design. The sha256 pin therefore guarantees (a) rebuild-vs-rebuild byte identity and (b) the
  released/CI-rebuilt product; the WORKING copy's guard is the per-table content-hash round trip
  (Task 7), not container bytes. (Spec §3.3 refined, not contradicted.)
- **D3 — sharding is a guarded contingency, not implemented code.** Per YAGNI + III.11: the
  exporter hard-fails at 1.8 GB/table. Contingency if tripped: shard by PK range into
  `<table>.NNN.parquet` with per-shard registry entries and builder concatenation in shard order —
  specced here so the escalation is concrete, implemented only if Phase 0 measurement demands it.
- **D4 — ceremony pre-authorization for Task 11** (see Global Constraints) is requested now so
  the cutover doesn't stall mid-flight.

## File Structure

- Create: `tools/measure_reference_export.py` — Phase 0 measurement CLI (kept as a diagnostic).
- Modify: `tools/make_data_artifacts.py` — extract `export_table_parquet()` helper; add
  `enumerate_full_coverage_specs()`, `--full-coverage`; manifest v2 writer (`schema` + `product`
  blocks) + `update_product_block()`.
- Create: `tools/extract_reference_schema.py` — canonical DDL extractor.
- Create: `tools/build_reference_db.py` — deterministic builder.
- Create: `tools/verify_reference_roundtrip.py` — per-table content comparison.
- Create: `tools/loader_to_sources.py` — loader→sources wrapper (Phase 6).
- Modify: `src/babylon/sentinels/coverage/checks.py` — `check_artifact_manifest` learns v2.
- Modify: `.mise.toml` — `data:artifacts`, `data:schema`, `data:build-db`, `data:verify-build`,
  `data:verify-roundtrip`, `data:subset` tasks.
- Modify: `.github/actions/fetch-reference-db/action.yml` — v7 pins + schema.sql fetch (cutover).
- Modify: `.github/workflows/nightly.yml` — rebuild-verification leg.
- Tests: `tests/unit/reference/test_measure_export.py`, extend
  `tests/unit/reference/test_data_artifacts.py`, `tests/unit/reference/test_schema_extract.py`,
  `tests/unit/reference/test_build_reference_db.py`, `tests/unit/reference/test_roundtrip.py`,
  extend the coverage-sentinel tests (find home via `rg -l check_artifact_manifest tests/`).

All unit tests in this plan run against SYNTHETIC temp SQLite fixtures (no reference DB, no
drive) so they live in the fast gate; every real-DB verification is a dev-box mise task or the
nightly refdata lane.

---

## Phase 0 — Measurement (read-only; the go/no-go on sharding)

### Task 1: Extract `export_table_parquet()` + measurement CLI

**Files:**
- Modify: `tools/make_data_artifacts.py`
- Create: `tools/measure_reference_export.py`
- Test: `tests/unit/reference/test_measure_export.py`

**Interfaces:**
- Produces: `export_table_parquet(conn: sqlite3.Connection, table: str, dest: Path) -> tuple[int, int]`
  (rows, bytes) in `make_data_artifacts.py` — the single parquet-writing path, used by Tasks 2/11
  and the measurement CLI. Also `governed_db_tables(conn) -> list[str]` (every
  `sqlite_master` table, `name NOT LIKE 'sqlite_%'`, in `ORDER BY name`).
- Consumes: existing `_table_layout()`, `PARQUET_COMPRESSION`, `PARQUET_COMPRESSION_LEVEL`,
  `_sha256()`.

- [ ] **Step 1: Write the failing tests** (import pattern: copy the `sys.path.insert(0,
  str(TOOLS_DIR))` header from `tests/unit/reference/test_data_artifacts.py`):

```python
def test_export_table_parquet_writes_deterministic_file(tmp_path):
    db = tmp_path / "mini.sqlite"
    conn = sqlite3.connect(db)
    conn.execute("CREATE TABLE dim_x (id INTEGER PRIMARY KEY, name TEXT, v REAL)")
    conn.executemany("INSERT INTO dim_x VALUES (?,?,?)", [(2, "b", 1.5), (1, "a", 0.5)])
    conn.commit()
    rows, size = export_table_parquet(conn, "dim_x", tmp_path / "a.parquet")
    rows2, size2 = export_table_parquet(conn, "dim_x", tmp_path / "b.parquet")
    assert rows == rows2 == 2
    assert _sha256(tmp_path / "a.parquet") == _sha256(tmp_path / "b.parquet")

def test_export_table_without_pk_sorts_by_all_columns(tmp_path):
    # A PK-less table must still export deterministically (sort key = all columns).
    db = tmp_path / "mini.sqlite"
    conn = sqlite3.connect(db)
    conn.execute("CREATE TABLE staging_y (a INTEGER, b TEXT)")
    conn.executemany("INSERT INTO staging_y VALUES (?,?)", [(2, "z"), (1, "y")])
    conn.commit()
    rows, _ = export_table_parquet(conn, "staging_y", tmp_path / "y.parquet")
    assert rows == 2
    tbl = pq.read_table(tmp_path / "y.parquet")
    assert tbl.column("a").to_pylist() == [1, 2]

def test_governed_db_tables_excludes_internal(tmp_path):
    conn = sqlite3.connect(tmp_path / "m.sqlite")
    conn.execute("CREATE TABLE fact_a (id INTEGER PRIMARY KEY AUTOINCREMENT, x TEXT)")
    conn.execute("INSERT INTO fact_a (x) VALUES ('q')")  # materializes sqlite_sequence
    conn.commit()
    assert governed_db_tables(conn) == ["fact_a"]
```

- [ ] **Step 2: Run to verify failure** — `mise run test:q -- tests/unit/reference/test_measure_export.py`
  → FAIL: `ImportError: cannot import name 'export_table_parquet'`.
- [ ] **Step 3: Implement in `make_data_artifacts.py`.** Refactor: locate the existing
  parquet-writing block inside the generate path (the code around the
  `compression=PARQUET_COMPRESSION, compression_level=PARQUET_COMPRESSION_LEVEL` call at ~line
  219) and extract it, preserving behavior byte-for-byte:

```python
def governed_db_tables(conn: sqlite3.Connection) -> list[str]:
    """Every user table in sqlite_master, name-sorted; internal sqlite_% excluded."""
    rows = conn.execute(
        "SELECT name FROM sqlite_master "
        "WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
    ).fetchall()
    return [r[0] for r in rows]


def export_table_parquet(conn: sqlite3.Connection, table: str, dest: Path) -> tuple[int, int]:
    """Export one table with the pipeline's determinism pins; returns (rows, bytes).

    PK-less tables sort by ALL columns (identical rows are interchangeable, so any
    stable total order yields byte-identical output).
    """
    columns, pk_cols, schema = _table_layout_or_all_columns(conn, table)
    order = ", ".join(f'"{c}"' for c in (pk_cols or columns))
    cols = ", ".join(f'"{c}"' for c in columns)
    rows = conn.execute(f'SELECT {cols} FROM "{table}" ORDER BY {order}').fetchall()
    arrays = [pa.array([r[i] for r in rows], type=schema.field(i).type) for i in range(len(columns))]
    pq.write_table(
        pa.Table.from_arrays(arrays, schema=schema),
        dest,
        compression=PARQUET_COMPRESSION,
        compression_level=PARQUET_COMPRESSION_LEVEL,
        row_group_size=max(len(rows), 1),
    )
    return len(rows), dest.stat().st_size
```

  `_table_layout_or_all_columns` wraps the existing `_table_layout` (which hard-errors on
  missing PK) and, on that specific error, retries treating all columns as the sort key —
  loudly logging `f"{table}: no PK — sorting by all columns"`. Re-route the existing generate
  path through `export_table_parquet` so there is exactly one writer (the existing
  double-generation test pins that this refactor changed nothing).
- [ ] **Step 4: Write `tools/measure_reference_export.py`:**

```python
#!/usr/bin/env python3
"""Phase-0 measurement for the parquet-canonical pipeline (plan 2026-07-19).

Read-only. Exports every governed table to a temp dir with the pipeline's exact pins,
records per-table rows/bytes plus the container pragmas the builder must pin, and writes
reports/parquet-size-measurement.json. Exits 2 (loud, III.11) if any table exceeds the
release-asset safety threshold — that triggers the plan's sharding contingency (D3).
"""
from __future__ import annotations

import argparse
import json
import sqlite3
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from make_data_artifacts import export_table_parquet, governed_db_tables  # noqa: E402

RELEASE_ASSET_LIMIT_BYTES = 2_000_000_000
SAFETY_FACTOR = 0.9
DEFAULT_DB = "data/sqlite/marxist-data-3NF.sqlite"
REPORT = Path("reports/parquet-size-measurement.json")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--db", default=DEFAULT_DB)
    args = parser.parse_args(argv)

    conn = sqlite3.connect(f"file:{args.db}?mode=ro", uri=True)
    pragmas = {
        name: conn.execute(f"PRAGMA {name}").fetchone()[0]
        for name in ("page_size", "application_id", "user_version", "encoding")
    }
    tables: list[dict[str, object]] = []
    threshold = int(RELEASE_ASSET_LIMIT_BYTES * SAFETY_FACTOR)
    oversized: list[str] = []
    with tempfile.TemporaryDirectory() as tmp:
        for table in governed_db_tables(conn):
            rows, size = export_table_parquet(conn, table, Path(tmp) / f"{table}.parquet")
            tables.append({"table": table, "rows": rows, "parquet_bytes": size})
            if size > threshold:
                oversized.append(table)
            print(f"  {table}: {rows} rows, {size:,} bytes")
    report = {
        "db": args.db,
        "sqlite_runtime": sqlite3.sqlite_version,
        "pragmas": pragmas,
        "threshold_bytes": threshold,
        "total_parquet_bytes": sum(int(t["parquet_bytes"]) for t in tables),  # type: ignore[arg-type]
        "tables": sorted(tables, key=lambda t: -int(t["parquet_bytes"])),  # type: ignore[arg-type]
    }
    REPORT.parent.mkdir(parents=True, exist_ok=True)
    REPORT.write_text(json.dumps(report, indent=2) + "\n")
    print(f"wrote {REPORT} (total {report['total_parquet_bytes']:,} bytes)")
    if oversized:
        print(f"OVERSIZED (> {threshold:,} bytes): {oversized} — sharding contingency D3 "
              "required; STOP and get plan addendum approved.", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
```

- [ ] **Step 5: Run unit tests green** — `mise run test:q -- tests/unit/reference/test_measure_export.py tests/unit/reference/test_data_artifacts.py` → PASS (both new tests AND the pre-existing double-generation test).
- [ ] **Step 6: Run the measurement on the dev box** (single-flight; ~4.5 GB read):
  `poetry run python tools/measure_reference_export.py` → expected exit 0; commit
  `reports/parquet-size-measurement.json`. **If exit 2: STOP the plan here** and take D3 to the
  owner.
- [ ] **Step 7: Commit** — `mise run commit -- "feat(data): export_table_parquet helper + phase-0 parquet size measurement"`; verify HEAD moved.

---

## Phase 1 — Full-coverage sources + registry v2

### Task 2: `enumerate_full_coverage_specs()` + `--full-coverage`

**Files:**
- Modify: `tools/make_data_artifacts.py`
- Test: extend `tests/unit/reference/test_data_artifacts.py`

**Interfaces:**
- Produces: `enumerate_full_coverage_specs(conn) -> tuple[ArtifactSpec, ...]`; CLI flag
  `--full-coverage` (default off until Task 10 flips the mise task).
- Consumes: `babylon.sentinels.coverage.catalog.load_catalog_tables()` (catalog rows carry the
  required `material_relation` — no hand-written strings for ~68 tables), existing `ARTIFACTS`
  tuple (curated entries keep their tier/format/home), `governed_db_tables`.

- [ ] **Step 1: Write the failing test:**

```python
def test_enumerate_full_coverage_specs(tmp_path, monkeypatch):
    conn = sqlite3.connect(tmp_path / "m.sqlite")
    conn.execute("CREATE TABLE fact_new (id INTEGER PRIMARY KEY, v REAL)")
    conn.execute("CREATE TABLE fact_energy_annual (id INTEGER PRIMARY KEY)")  # curated already
    conn.commit()
    fake_catalog = {"fact_new": SimpleNamespace(material_relation="test relation")}
    monkeypatch.setattr(mda, "_catalog_by_name", lambda: fake_catalog)
    specs = mda.enumerate_full_coverage_specs(conn)
    assert [s.name for s in specs] == ["fact_new"]          # curated table skipped
    assert specs[0].home == "dist/data-artifacts/fact_new.parquet"
    assert specs[0].material_relation == "test relation"
    assert specs[0].mode == "generate"

def test_enumerate_full_coverage_missing_catalog_row_is_loud(tmp_path, monkeypatch):
    conn = sqlite3.connect(tmp_path / "m.sqlite")
    conn.execute("CREATE TABLE fact_orphan (id INTEGER PRIMARY KEY)")
    conn.commit()
    monkeypatch.setattr(mda, "_catalog_by_name", dict)
    with pytest.raises(KeyError, match="fact_orphan"):
        mda.enumerate_full_coverage_specs(conn)
```

- [ ] **Step 2: Run to verify failure** → FAIL: `AttributeError: ... enumerate_full_coverage_specs`.
- [ ] **Step 3: Implement:**

```python
def _catalog_by_name() -> dict[str, object]:
    from babylon.sentinels.coverage.catalog import load_catalog_tables

    return {t.name: t for t in load_catalog_tables()}


def enumerate_full_coverage_specs(conn: sqlite3.Connection) -> tuple[ArtifactSpec, ...]:
    """One parquet spec per DB table not already curated in ARTIFACTS.

    material_relation is inherited from the table's data-catalog.yaml row (Aleksandrov
    trace); a DB table with no catalog row raises KeyError loudly — the catalog
    sentinel's bijection makes that state a bug, never a default.
    """
    catalog = _catalog_by_name()
    curated = {spec.source_table for spec in ARTIFACTS}
    specs = []
    for table in governed_db_tables(conn):
        if table in curated:
            continue
        if table not in catalog:
            raise KeyError(f"{table}: DB table has no data-catalog.yaml row — fix the catalog first")
        specs.append(
            ArtifactSpec(
                name=table,
                format="parquet",
                source_table=table,
                home=f"dist/data-artifacts/{table}.parquet",
                material_relation=catalog[table].material_relation,
                mode="generate",
            )
        )
    return tuple(specs)
```

  Wire `--full-coverage` into the CLI: when set, the working spec list is
  `ARTIFACTS + enumerate_full_coverage_specs(conn)`; default path unchanged.
- [ ] **Step 4: Run green** — `mise run test:q -- tests/unit/reference/test_data_artifacts.py` → PASS.
- [ ] **Step 5: Commit** — `mise run commit -- "feat(data): full-coverage artifact enumeration from the catalog"`.

### Task 3: Manifest v2 (`schema` + `product` blocks) + sentinel update

**Files:**
- Modify: `tools/make_data_artifacts.py`
- Modify: `src/babylon/sentinels/coverage/checks.py`
- Test: extend `tests/unit/reference/test_data_artifacts.py` + the coverage-sentinel test file
  (locate: `rg -l check_artifact_manifest tests/` — single expected hit).

**Interfaces:**
- Produces: `_write_manifest(entries, *, schema_entry=None, product_entry=None)` emitting
  `version: "2.0.0"`; `update_product_block(manifest_path: Path, product: dict[str, object])`
  (read-modify-rewrite through the same writer — one writer function, two callers: exporter and
  builder). Manifest v2 shape:

```yaml
version: "2.0.0"
schema:                       # written by Task 4's extractor run
  file: dist/data-artifacts/schema.sql
  sha256: <hex>
  tables: 76
  views: 8
  indexes: 100
product:                      # written ONLY by tools/build_reference_db.py
  name: marxist-data-3NF.sqlite
  sha256: <hex>               # rebuild-vs-rebuild pin; see plan D2
  page_size: 4096
  application_id: 0x4241424C
  user_version: 1
  sqlite_version: "3.46.1"
artifacts: [ ...existing entry shape, unchanged... ]
```

- [ ] **Step 1: Failing tests** — round-trip: `_write_manifest` with both blocks → parsed YAML
  has `version == "2.0.0"`, blocks intact, artifact entries byte-identical to v1 formatting;
  `update_product_block` on a manifest WITHOUT a product block adds one and leaves `artifacts`
  bytes unchanged; `check_artifact_manifest` (a) still passes on the current committed v1 file,
  (b) passes on a v2 fixture whose `schema.file` is absent locally when it points into `dist/`
  (skip-when-absent, same rule as dist-tier artifacts), (c) FAILS on a v2 fixture whose in-repo
  schema file exists but hash-mismatches.
- [ ] **Step 2: Run to verify failures.**
- [ ] **Step 3: Implement** (writer emits blocks in fixed order `version, schema, product,
  artifacts`; `product.sha256` is advisory metadata to the static check — it is verified by the
  builder and CI, never by the fast gate, per D2).
- [ ] **Step 4: Run green** — scoped tests + `poetry run python tools/sentinel_check.py coverage --check` (current v1 manifest still passes).
- [ ] **Step 5: Commit** — `mise run commit -- "feat(data): data-artifacts manifest v2 (schema+product blocks) + sentinel support"`.

---

## Phase 2 — Canonical DDL

### Task 4: `tools/extract_reference_schema.py`

**Files:**
- Create: `tools/extract_reference_schema.py`
- Test: `tests/unit/reference/test_schema_extract.py`

**Interfaces:**
- Produces: `extract_schema_sql(conn) -> str` (canonical DDL text) and
  `schema_census(conn) -> dict[str, int]` (`{"tables": n, "views": n, "indexes": n}`); CLI writes
  `dist/data-artifacts/schema.sql` + updates the manifest `schema` block via Task 3's writer.
- Canonical form: statements in `sqlite_master` **rowid order** (that IS "sqlite_master order");
  per-statement canonicalization = strip trailing whitespace per line, exactly one trailing `;`,
  one blank line between statements, trailing newline; `sqlite_%` internal objects and NULL-sql
  rows (auto-indexes) excluded.

- [ ] **Step 1: Failing tests:**

```python
def _mini_db(tmp_path):
    conn = sqlite3.connect(tmp_path / "m.sqlite")
    conn.executescript(
        "CREATE TABLE fact_a (id INTEGER PRIMARY KEY, v REAL);\n"
        "CREATE INDEX idx_fact_a_v ON fact_a (v);\n"
        "CREATE VIEW view_a AS SELECT id FROM fact_a;\n"
    )
    return conn

def test_extract_preserves_master_order_and_terminators(tmp_path):
    text = extract_schema_sql(_mini_db(tmp_path))
    stmts = [s for s in text.split(";\n") if s.strip() and not s.lstrip().startswith("--")]
    assert "CREATE TABLE fact_a" in stmts[0]
    assert "CREATE INDEX idx_fact_a_v" in stmts[1]
    assert "CREATE VIEW view_a" in stmts[2]
    assert text.endswith(";\n")

def test_extract_is_idempotent_through_rebuild(tmp_path):
    conn = _mini_db(tmp_path)
    text = extract_schema_sql(conn)
    conn2 = sqlite3.connect(tmp_path / "n.sqlite")
    conn2.executescript(text)
    assert extract_schema_sql(conn2) == text  # fixed point: DDL survives a round trip

def test_every_view_is_present(tmp_path):
    conn = _mini_db(tmp_path)
    views = [r[0] for r in conn.execute("SELECT name FROM sqlite_master WHERE type='view'")]
    text = extract_schema_sql(conn)
    assert views and all(f"CREATE VIEW {v}" in text for v in views)
```

  The fixed-point test is the generic form of the spec's "verify against the Fundamental Theorem
  views specifically": at cutover (Task 10 step 3) the SAME assertion runs against the real DB,
  which covers all 8 real views by construction rather than by a hardcoded name list.
- [ ] **Step 2: Run to verify failure** → FAIL: module not found.
- [ ] **Step 3: Implement:**

```python
HEADER = (
    "-- schema.sql — canonical DDL for the reference DB (parquet-canonical pipeline).\n"
    "-- GENERATED by tools/extract_reference_schema.py — do not edit by hand.\n"
    "-- Statement order = sqlite_master rowid order of the source DB.\n"
)


def extract_schema_sql(conn: sqlite3.Connection) -> str:
    rows = conn.execute(
        "SELECT sql FROM sqlite_master "
        "WHERE sql IS NOT NULL AND name NOT LIKE 'sqlite_%' ORDER BY rowid"
    ).fetchall()
    statements = []
    for (sql,) in rows:
        body = "\n".join(line.rstrip() for line in sql.strip().splitlines())
        statements.append(body.rstrip(";") + ";")
    return HEADER + "\n" + "\n\n".join(statements) + "\n"


def schema_census(conn: sqlite3.Connection) -> dict[str, int]:
    counts = dict(
        conn.execute(
            "SELECT type, COUNT(*) FROM sqlite_master "
            "WHERE sql IS NOT NULL AND name NOT LIKE 'sqlite_%' "
            "GROUP BY type"
        ).fetchall()
    )
    return {
        "tables": counts.get("table", 0),
        "views": counts.get("view", 0),
        "indexes": counts.get("index", 0),
    }
```

  CLI: `--db` (default the reference path), `--out dist/data-artifacts/schema.sql`; writes the
  file, computes sha256 with `make_data_artifacts._sha256`, calls the manifest writer with the
  populated `schema` block (census counts included).
- [ ] **Step 4: Run green** — `mise run test:q -- tests/unit/reference/test_schema_extract.py`.
- [ ] **Step 5: Commit** — `mise run commit -- "feat(data): canonical schema.sql extractor"`.

---

## Phase 3 — The builder

### Task 5: `tools/build_reference_db.py`

**Files:**
- Create: `tools/build_reference_db.py`
- Test: `tests/unit/reference/test_build_reference_db.py`

**Interfaces:**
- Produces:

```python
PAGE_SIZE = 4096                 # measured on the live DB (Phase 0 report)
APPLICATION_ID = 0x4241424C      # "BABL"
USER_VERSION = 1                 # pipeline major version
PINNED_SQLITE_VERSION = "3.46.1" # bump = declared regeneration event

@dataclasses.dataclass(frozen=True)
class BuildResult:
    path: Path
    sha256: str
    sqlite_version: str
    tables_built: int
    rows_inserted: int

def build_reference_db(
    manifest_path: Path, schema_sql_path: Path, sources_root: Path, out_path: Path
) -> BuildResult: ...
```

- Consumes: manifest v2 (Task 3), `schema.sql` (Task 4), parquet/CSV sources; `_sha256` from
  `make_data_artifacts`.
- Build algorithm (each point is a determinism decision, all fixed):
  1. Assert `sqlite3.sqlite_version == PINNED_SQLITE_VERSION`, else raise `BuildEnvironmentError`.
  2. Fresh file (unlink first); `PRAGMA page_size`, `PRAGMA journal_mode=DELETE` before any DDL.
  3. Split `schema.sql` into statements; classify by leading keywords (`CREATE TABLE` /
     `CREATE [UNIQUE] INDEX` / `CREATE VIEW` / `CREATE TRIGGER`); an unclassifiable statement is
     a hard error. Index→table via `re.search(r'\bON\s+"?(\w+)"?', stmt, re.IGNORECASE)`, hard
     error on no match.
  4. Execute all CREATE TABLE statements in file order.
  5. For every manifest artifact entry **whose `source_table` has a CREATE TABLE in schema.sql**
     (this membership test is what distinguishes DB sources from artifact-only demoted tables —
     no new registry field needed), in manifest order: verify source file sha256 against the
     manifest (hard error on mismatch), stream rows in file order
     (`pq.ParquetFile(...).iter_batches(65536)` for parquet; `csv.reader` for csv entries with
     NULL/type coercion mirroring the exporter), `executemany` INSERT; then execute that table's
     CREATE INDEX statements in file order. A schema table with NO manifest entry is a hard
     error (coverage hole), and vice-versa is already covered by the membership test.
  6. Execute CREATE VIEW (and any trigger) statements in file order.
  7. `PRAGMA application_id`, `PRAGMA user_version`; commit; `VACUUM`; close.
  8. Return `BuildResult` with the file's sha256.

- [ ] **Step 1: Failing tests** (synthetic end-to-end through the real tools):

```python
def _make_source_db(tmp_path):
    conn = sqlite3.connect(tmp_path / "src.sqlite")
    conn.executescript(
        "CREATE TABLE dim_k (id INTEGER PRIMARY KEY, name TEXT);\n"
        "CREATE TABLE fact_m (id INTEGER PRIMARY KEY, k_id INTEGER, v REAL);\n"
        "CREATE INDEX idx_fact_m_k ON fact_m (k_id);\n"
        "CREATE VIEW view_m AS SELECT k_id, SUM(v) s FROM fact_m GROUP BY k_id;\n"
    )
    conn.executemany("INSERT INTO dim_k VALUES (?,?)", [(1, "a"), (2, "b")])
    conn.executemany("INSERT INTO fact_m VALUES (?,?,?)", [(1, 1, 2.5), (2, 2, 4.0)])
    conn.commit()
    return conn

def test_double_build_is_byte_identical(tmp_path):
    conn = _make_source_db(tmp_path)
    manifest, schema = _export_all(conn, tmp_path)   # helper: export_table_parquet each table,
                                                     # extract_schema_sql, write manifest v2
    r1 = build_reference_db(manifest, schema, tmp_path, tmp_path / "out1.sqlite")
    r2 = build_reference_db(manifest, schema, tmp_path, tmp_path / "out2.sqlite")
    assert r1.sha256 == r2.sha256

def test_build_matches_source_content(tmp_path):
    conn = _make_source_db(tmp_path)
    manifest, schema = _export_all(conn, tmp_path)
    r = build_reference_db(manifest, schema, tmp_path, tmp_path / "out.sqlite")
    out = sqlite3.connect(r.path)
    assert out.execute("SELECT * FROM dim_k ORDER BY id").fetchall() == [(1, "a"), (2, "b")]
    assert out.execute("SELECT s FROM view_m ORDER BY k_id").fetchall() == [(2.5,), (4.0,)]
    assert out.execute("PRAGMA application_id").fetchone()[0] == APPLICATION_ID

def test_source_hash_mismatch_is_fatal(tmp_path): ...   # corrupt one parquet byte → SourceHashError
def test_schema_table_without_source_is_fatal(tmp_path): ...  # drop a manifest entry → CoverageHoleError
def test_wrong_sqlite_version_is_fatal(monkeypatch): ...      # monkeypatch sqlite3.sqlite_version
```

- [ ] **Step 2: Run to verify failure.**
- [ ] **Step 3: Implement** per the algorithm above. Exceptions are specific
  (`BuildEnvironmentError`, `SourceHashError`, `CoverageHoleError`, `SchemaParseError`), all
  subclassing a module-local `ReferenceBuildError` — no generic catches.
- [ ] **Step 4: Run green** — `mise run test:q -- tests/unit/reference/test_build_reference_db.py`.
- [ ] **Step 5: Commit** — `mise run commit -- "feat(data): deterministic reference-DB builder with double-build proof"`.

### Task 6: Dev-box full double-build task

- [ ] **Step 1:** Add to `.mise.toml` (with the other data tasks):

```toml
[tasks."data:verify-build"]
description = "Build the reference DB twice from sources and prove byte-identity (dev box only)"
run = """
poetry run python tools/build_reference_db.py --out dist/build/ref-a.sqlite
poetry run python tools/build_reference_db.py --out dist/build/ref-b.sqlite
python3 - <<'EOF'
import hashlib, pathlib, sys
h = [hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
     for p in ("dist/build/ref-a.sqlite", "dist/build/ref-b.sqlite")]
print(h[0]); sys.exit(0 if h[0] == h[1] else 1)
EOF
"""
```

  (This task is exercised for real in Task 10; here it only needs to parse — `mise tasks | rg
  data:verify-build` → listed.)
- [ ] **Step 2: Commit** with Task 8's mise work if convenient, else
  `mise run commit -- "feat(data): data:verify-build double-build gate"`.

---

## Phase 4 — Content round trip

### Task 7: `tools/verify_reference_roundtrip.py`

**Files:**
- Create: `tools/verify_reference_roundtrip.py`
- Test: `tests/unit/reference/test_roundtrip.py`

**Interfaces:**
- Produces: `table_content_hash(conn, table) -> tuple[int, str]` (row count, sha256 over
  canonicalized ordered rows) and `compare_databases(live, rebuilt) -> RoundtripReport`
  (per-table verdicts; `ok: bool`). CLI exits 1 on any mismatch, printing every differing table.
- **Value canonicalization (the D2/NUMERIC rule, stated once, tested):** for columns whose
  declared type maps to float64 in `_DECLTYPE_TO_ARROW`, hash `quote(CAST(col AS REAL))`;
  all other columns hash `quote(col)`. This makes the declared NUMERIC→float64 normalization
  invisible to the comparison (storage-class drift between INTEGER 1 and REAL 1.0 in a NUMERIC
  column is the documented, accepted transformation) while any VALUE change stays loud. Views are
  hashed with plain `quote()` on all columns (views carry no decltypes) after `ORDER BY` all
  columns. Internal `sqlite_%` tables are excluded (their content is build-history, not data).

- [ ] **Step 1: Failing tests:** identical synthetic DBs → `ok is True`; mutate one value →
  that table (and only that table) reported; NUMERIC column holding INTEGER 1 in one DB and REAL
  1.0 in the other → **equal** (canonicalization test); extra row → row-count mismatch reported.
- [ ] **Step 2: Run to verify failure.**
- [ ] **Step 3: Implement** (streaming: hash row-by-row over a server-side cursor; never
  materialize a full table).
- [ ] **Step 4: Run green** — `mise run test:q -- tests/unit/reference/test_roundtrip.py`.
- [ ] **Step 5: Commit** — `mise run commit -- "feat(data): reference-DB content round-trip verifier"`.

---

## Phase 5 — Wiring (all inert until the v7 release exists)

### Task 8: mise tasks

- [ ] **Step 1:** Add to `.mise.toml`:

```toml
[tasks."data:artifacts"]
description = "Regenerate ALL reference sources (full-coverage parquet + manifest v2) from the live DB"
run = "poetry run python tools/make_data_artifacts.py --full-coverage"

[tasks."data:schema"]
description = "Extract canonical schema.sql from the live DB + update the manifest schema block"
run = "poetry run python tools/extract_reference_schema.py"

[tasks."data:build-db"]
description = "Rebuild the reference DB from parquet sources + schema.sql (product block updated)"
run = "poetry run python tools/build_reference_db.py --out dist/build/marxist-data-3NF.sqlite --update-manifest"

[tasks."data:verify-roundtrip"]
description = "Content-compare the live reference DB against the rebuilt product"
run = "poetry run python tools/verify_reference_roundtrip.py --live data/sqlite/marxist-data-3NF.sqlite --rebuilt dist/build/marxist-data-3NF.sqlite"

[tasks."data:subset"]
description = "Build the CI subset FROM THE REBUILT PRODUCT (subset drift dies by construction)"
run = "poetry run python tools/make_reference_subset.py --source dist/build/marxist-data-3NF.sqlite --output dist/build/reference-subset.sqlite --manifest dist/build/reference-subset.manifest.json"
```

- [ ] **Step 2:** `mise tasks | rg 'data:(artifacts|schema|build-db|verify-roundtrip|subset)'` →
  all five listed. Update `ai/tooling.yaml` with the five tasks.
- [ ] **Step 3: Commit** — `mise run commit -- "feat(tooling): parquet-pipeline mise tasks (artifacts/schema/build/verify/subset)"`.

### Task 9: CI rebuild-verification leg

**Files:**
- Modify: `.github/workflows/nightly.yml` (refdata lane)
- Modify: `.github/actions/fetch-reference-db/action.yml` (schema.sql fetch only; pins stay v6
  until cutover)

- [ ] **Step 1:** In `fetch-reference-db/action.yml`, extend the existing manifest-driven
  dist-tier fetch loop so `schema:` block files (`dist/...` homes) are fetched + sha-verified
  exactly like artifact entries (the manifest commit remains the pin bump — no new action
  inputs).
- [ ] **Step 2:** In `nightly.yml`'s `refdata-tests` job, append a `Rebuild-verify reference DB`
  step, guarded so it no-ops (with a loud notice) while the manifest has no `product` block —
  which keeps the leg inert until the cutover commit lands:

```yaml
      - name: Rebuild-verify reference DB (parquet-canonical gate)
        run: |
          if ! rg -q '^product:' data-artifacts.yaml; then
            echo "::notice::no product block in data-artifacts.yaml yet — rebuild-verify inert (pre-cutover)"
            exit 0
          fi
          poetry run python tools/build_reference_db.py --out /tmp/rebuilt.sqlite
          poetry run python - <<'EOF'
          import hashlib, pathlib, sys, yaml
          want = yaml.safe_load(pathlib.Path("data-artifacts.yaml").read_text())["product"]["sha256"]
          got = hashlib.sha256(pathlib.Path("/tmp/rebuilt.sqlite").read_bytes()).hexdigest()
          print(f"registry={want}\nrebuilt ={got}")
          sys.exit(0 if want == got else 1)
          EOF
```

  Record the step's wall-clock on its first real (post-cutover) run; if it exceeds 20 minutes,
  move the step to the weekly `sim-artifacts` job in a follow-up commit (documented fallback,
  decided by measurement, not now).
- [ ] **Step 3:** Sanity: `poetry run python -c "import yaml; yaml.safe_load(open('.github/workflows/nightly.yml'))"` → no error, and push triggers workflow-lint in CI.
- [ ] **Step 4: Commit** — `mise run commit -- "ci(data): nightly rebuild-verification leg (inert until product block lands)"`.

**End of Phase 5. Everything above is qa-neutral, CI-inert, and mergeable as one PR to dev.
The gate before Phase 6: owner has reviewed THIS plan, the Phase-0 measurement report, and the
Phases 0–5 PR. Phase 6 then starts on a fresh branch with nothing else in flight.**

---

## Phase 6 — CUTOVER (owner-gated; runs ALONE; one branch, sequenced commits)

### Task 10: Export, verify, flip authority

Runbook — each step is a checkpoint; any red STOPS the cutover with the working DB untouched:

- [ ] **Step 1: Preflight.** `git worktree list` shows only main/vol3/known worktrees; no other
  branch mid-flight touches reference data; announce cutover start in the PR. `mise run
  data:doctor` green.
- [ ] **Step 2: Export.** `mise run data:artifacts` then `mise run data:schema` (full-coverage
  parquet into `dist/data-artifacts/`, manifest v2 with schema block). Commit the manifest:
  `mise run commit -- "feat(data): full-coverage parquet export + schema.sql (parquet-canonical sources)"`.
- [ ] **Step 3: Schema fixed-point check on the real DB.** Rebuild-extract:
  `poetry run python tools/build_reference_db.py --out dist/build/marxist-data-3NF.sqlite --update-manifest`
  then `poetry run python tools/extract_reference_schema.py --db dist/build/marxist-data-3NF.sqlite --out /tmp/schema-rt.sql`
  and `diff dist/data-artifacts/schema.sql /tmp/schema-rt.sql` → empty (all 8 views included by
  construction). Commit the product block:
  `mise run commit -- "feat(data): first pinned reference-DB build product"`.
- [ ] **Step 4: Proofs.** `mise run data:verify-build` (double-build byte identity) →
  identical hashes; `mise run data:verify-roundtrip` (content vs live DB) → `ok`. Both outputs
  pasted into the PR.
- [ ] **Step 5: Simulation gate BEFORE the flip.**
  `BABYLON_NORMALIZED_DB_PATH="$PWD/dist/build/marxist-data-3NF.sqlite" mise run qa:regression`
  → **5/5 byte-identical**. Any drift = STOP (the rebuild changed semantics — investigate the
  roundtrip report; do not proceed).
- [ ] **Step 6: The flip.** Backup then replace, on the drive:

```bash
cp /media/user/data/babylon-data/sqlite/marxist-data-3NF.sqlite \
   /media/user/data/babylon-data/backups/marxist-data-3NF.pre-parquet-$(date +%Y%m%d).sqlite
cp dist/build/marxist-data-3NF.sqlite /media/user/data/babylon-data/sqlite/marxist-data-3NF.sqlite
```

  Then `mise run qa:regression` (now via the normal symlink path) → 5/5;
  `poetry run python tools/sentinel_check.py catalog --check` → green (bijection + KEEP-emptiness
  against the rebuilt DB).
- [ ] **Step 7: Release ci-data-v7.** `mise run data:subset` (from the rebuilt product);
  `gh release create ci-data-v7` uploading: the subset (asset name
  `reference-subset-<date>-v7.sqlite`), ALL `dist/data-artifacts/*.parquet`, `schema.sql`, and
  the TIGER tarball carried over from v6. Update
  `.github/actions/fetch-reference-db/action.yml` defaults (`tag`, `asset`, `sha256`, changelog
  comment: "v7 = parquet-canonical cutover; subset now DERIVED from parquet sources; adds full
  per-table parquet + schema.sql"). Commit:
  `mise run commit -- "ci(data): ci-data-v7 — parquet-canonical release + pin bump"`.
- [ ] **Step 8: Belt-and-braces.** Copy the canonical source set to the drive archive:
  `cp -r dist/data-artifacts /media/user/data/babylon-data/backups/data-artifacts-v7`.
- [ ] **Step 9:** `mise run check` green; dispatch the nightly workflow once
  (`gh workflow run nightly.yml --ref <branch>`) and confirm the rebuild-verify leg goes green
  for real (it is no longer inert — the product block exists).

### Task 11: IMPORT_USE lands as the first source-only ingest

**Files:**
- Create: `tools/loader_to_sources.py`
- Modify: `tools/ingest_bea_imports.py` (docstring only: usage now goes through the wrapper)
- Test: `tests/unit/reference/test_loader_to_sources.py`

**Interfaces:**
- Produces:

```python
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
    """
```

- [ ] **Step 1: Failing test** (synthetic): a toy loader_main that inserts one row into a
  scratch DB → wrapper re-exports the affected table, manifest hash for it changes, all other
  entries' hashes unchanged, scratch file gone, original product file byte-unchanged.
- [ ] **Step 2: Run to verify failure; implement; run green.** Implementation: copy product →
  `tempfile.NamedTemporaryFile`; call `loader_main(["--db-url", f"sqlite:///{scratch}"])`
  (the existing `ingest_bea_imports.main(argv)` signature takes exactly this); re-export each
  affected table via `export_table_parquet`; regenerate manifest via the Task 3 writer.
- [ ] **Step 3: Run the real ingest to sources** (dev box):

```bash
poetry run python tools/loader_to_sources.py \
    --loader ingest_bea_imports \
    --tables fact_bea_io_coefficient,dim_bea_io_table_type,dim_time
```

  Expected stdout: per-year "Ingested N import use coefficients" lines from the loader, then the
  re-export summary. `dim_time` is expected UNCHANGED (years 2010–2024 already exist from USE);
  the wrapper reporting it changed is a red flag to investigate before committing.
- [ ] **Step 4: Rebuild + gates.** `mise run data:build-db` → new product sha (declared in the
  commit); `mise run data:verify-build` → identical double-build; flip the working DB (same
  backup+cp as Task 10 step 6); `mise run qa:regression`:
  - 5/5 byte-identical → commit plainly.
  - Drift → **this is the pre-authorized ceremony**: regenerate baselines, commit as
    `test(baselines): IMPORT_USE landing — Φ path gains real import-penetration data` with a
    per-scenario drift table (which values moved, why m_j now non-zero moves them) in the commit
    body. Owner audits in the cutover PR.
- [ ] **Step 5: Φ consumers light up.** Locate the KNOWN-RED test:
  `rg -ln "phi_hour" tests/integration/` (expected: one file, marked known-red per the
  2026-07-18 audit) — run it scoped; it must now PASS on real IMPORT_USE data; remove its
  known-red marking in the same commit. Also run
  `mise run test:q -- tests/integration/economics/` and report the delta of the ~34
  pre-existing NoDataSentinel failures (any that flip green get listed in the PR body; none may
  flip red).
- [ ] **Step 6: Commit** — `mise run commit -- "feat(data): IMPORT_USE lands as a parquet source via loader_to_sources"` (+ the ceremony commit if step 4 drifted). Re-release: upload the
  changed parquet + subset to ci-data-v7 (assets are replaceable pre-announcement) or cut v8 if
  v7 was already consumed by a green nightly — decide by whether the v7 nightly ran; document
  the choice in the PR.

### Task 12: Governance close-out

- [ ] **Step 1: ADR.** Add `ai/decisions/ADR0NN_parquet_canonical_reference.yaml` (next free
  number per `ai/decisions/index.yaml`, updating the index): records the inversion, the D1–D4
  deviations as ruled, the pins, and the layered contract (file hash = drift alarm on the build
  product; tick hash = the invariant; content hash = working-copy guard).
- [ ] **Step 2: Docs.** `docs/how-to/reference-data-pipeline.rst` (how to add data: write/emit a
  parquet source + catalog row → `mise run data:build-db` — the loader_to_sources template;
  single Diataxis quadrant: how-to). Update the stale census note in
  `src/babylon/reference/schema.py`'s module docstring to name the manifest as lineage authority.
  Update `CLAUDE.md`'s Ledger bullet (reference DB is now a build product) and
  `ai/architecture.yaml`'s data section. Update `docs/superpowers/specs/2026-07-18-parquet-reference-pipeline-design.md` status line to "EXECUTED".
- [ ] **Step 3: State.** Update `ai/state.yaml`; append the memory file
  (`parquet-reference-pipeline-program.md`) with the cutover facts + gotchas discovered.
- [ ] **Step 4: Commit** — `mise run commit -- "docs(adr): ADR0NN parquet-canonical reference pipeline executed"`.

---

## Acceptance gates (from the spec §5, mapped to tasks)

1. Double-build byte-stability green on dev box AND in CI → Tasks 5/6/10.4, 9 (+10.9).
2. Content round-trip proven for all tables + views → Tasks 7, 10.4.
3. `qa:regression` 5/5 byte-identical against the rebuilt DB → Task 10.5/10.6 (Task 11.4 may be
   a declared ceremony instead — D4).
4. Catalog + coverage sentinels green with the expanded registry → Tasks 2/3, 10.6.
5. IMPORT_USE in the rebuilt DB; dark Φ consumers light up → Task 11.5.
6. CI reference legs run against rebuilt-and-verified data → Task 9 + 10.7/10.9.

## Self-review notes (writing-plans discipline)

- Spec coverage: §2 inversion → Tasks 2–5, 10; §3.1 → Tasks 2–4; §3.2 → Tasks 5–6; §3.3 → Task 9
  + D2; §3.4 → Tasks 8–9, 10.7; §3.5 → Task 11 (+ D1); §4 → phase gating + Task 10 ordering + 12;
  §5 → gates table above; §6 → untouched.
- The two spec ambiguities found became explicit decisions: D1 (IMPORT_USE table identity — the
  spec contradicts the written loader; plan follows the code) and D2 (container-hash scope under
  WAL). Both are surfaced for the owner rather than silently chosen — approval of this plan
  ratifies them.
- Type consistency: `export_table_parquet` (Task 1) is the single writer consumed by Tasks 2, 7
  (via `_DECLTYPE_TO_ARROW`), and 11; manifest v2 writer (Task 3) is the single YAML writer
  consumed by Tasks 4, 5 (`--update-manifest`), and 11.
