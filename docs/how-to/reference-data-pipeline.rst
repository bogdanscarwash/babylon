How to add or change reference data (parquet-canonical pipeline)
================================================================

.. note::
   Since the Phase-6 cutover (ADR098, 2026-07-20) the canonical reference
   sources are per-table parquet files plus ``schema.sql``, both registered in
   ``data-artifacts.yaml``. The SQLite file ``data/sqlite/marxist-data-3NF.sqlite``
   is a **build product** — never edit it in place.

Prerequisites
-------------

- Python 3.12.14 linked to SQLite 3.53.1 for reference-data reproduction.
- The data drive mounted (``mise run data:doctor`` green) for drive-sourced
  ingests.

The development environment on Linux x64 includes the governed SQLite version.
``mise.lock`` pins Python 3.12.14 from 20260901 by its
download URL and SHA-256. Install the same tools and dependencies used for
ordinary development::

    mise install --locked
    mise run install
    mise exec -- uv run --frozen python -c "import sqlite3; print(sqlite3.sqlite_version)"

The builder refuses other SQLite versions. Reference-data commands use
the ordinary project ``.venv`` and frozen ``uv.lock`` dependencies.

``data:build-db`` rebuilds from the registered sources. ``data:verify-build``
checks two builds for byte identity. Existing source and product checks remain
mandatory; a dependency update does not authorize an artifact-pin change.

Add a new table
---------------

1. Add the table's DDL to the schema by creating it in a scratch build and
   re-extracting — the canonical DDL is whatever
   ``tools/extract_reference_schema.py`` emits; ``schema.sql`` must be the
   fixed point of build→extract.
2. Add a catalog row in ``data-catalog.yaml`` (the per-table lineage registry;
   the catalog sentinel enforces catalog↔DB bijection).
3. Emit the table's parquet into ``dist/data-artifacts/`` and register it:
   ``mise run data:artifacts`` regenerates the manifest with per-file sha256
   pins.
4. Rebuild and verify::

       mise run data:build-db        # deterministic rebuild from sources
       mise run data:verify-build    # double-build byte identity
       mise run data:verify-roundtrip

5. Run ``mise run qa:regression`` — byte-identical, or STOP.

Change rows in an existing table (ingest)
-----------------------------------------

Loaders produce **sources**; only the builder produces the DB. Run any legacy
DB-writing loader through the wrapper::

    mise exec -- uv run --frozen python tools/loader_to_sources.py \
        --loader <module_name_in_tools> \
        --tables <comma-separated affected tables>

The wrapper copies the build product to a scratch file, runs the loader
against the scratch (``--db-url sqlite:///<scratch>``), re-exports each
affected table as parquet, regenerates the manifest, and deletes the scratch.
The shared DB is never opened for write. A loader that exits nonzero aborts
loudly with nothing changed.

Then rebuild + verify as above, and flip the working DB only after
``qa:regression`` is green (backup first — see the ADR098 flip procedure).
If baselines move, that is a declared ceremony: ``test(baselines):`` commit
with a drift table and a ``Baselines: blessed(<slug>)`` trailer.

Second-order artifacts (outside the build fixed point)
--------------------------------------------------------

Some registered artifacts are **not** part of the ``schema.sql`` + parquet
fixed point ``data:build-db`` rebuilds. They derive from the *registered*
parquet sources (``dist/data-artifacts/*.parquet``, sha-verified before any
read), never from the SQLite build product or the live DB directly, and they
back no table in ``schema.sql``. ``county_fips_vintage_crosswalk``,
``national_incidence_county_pole``, and ``national_reproduction_floor``
(``tools/make_fips_vintage_crosswalk.py`` and
``tools/make_national_incidence_artifact.py``, #334 Phase 0, ADR098/ADR171)
are the current instances, alongside the existing LODES/FAF/MIT-Election-Lab
class of hand-maintained entries in ``data-artifacts.yaml``'s ``EXCEPTION``
tail.

Two properties distinguish this class:

- **Non-interference with the build fixed point.** Regenerating a
  second-order artifact touches no schema and no registered parquet source,
  so ``mise run data:build-db`` followed by ``mise run data:verify-roundtrip``
  reproduces the exact same reference-DB product sha before and after.
- **Double-run byte identity is the whole reproducibility contract.** With no
  ``schema.sql`` table backing them, these artifacts have nothing for
  ``data:verify-build``'s double-build comparison to check — running the
  generator twice from clean and comparing sha256 is the only reproducibility
  proof that applies. Regenerate::

      mise run data:national-incidence  # A2 + A3
      mise exec -- uv run --frozen python tools/make_fips_vintage_crosswalk.py  # A1

Each artifact's ``data-artifacts.yaml`` entry is hand-registered — a real
``tools/make_data_artifacts.py`` regeneration (no ``--check``) never touches
it, since these names live outside its ``ARTIFACTS`` tuple. A regeneration
tripwire test per artifact (e.g.
``tests/unit/tools/test_national_incidence_manifest_entries.py``) turns a
silent manifest wipe into a failing test.

Gotchas (hard-won at the cutover)
---------------------------------

- **Never hand-type a sha256** — extract pins programmatically from the
  manifest and compare computed-vs-computed.
- The working copy's container bytes change on first open (WAL) **by
  design** — the container sha pins the *build product*; the working copy's
  guard is the per-table content-hash roundtrip.
- ``ingest_bea_imports`` (and loaders of its era) are one-shot, not
  idempotent: re-running against data that already contains their rows aborts
  on UNIQUE keys. Check the target table first.
- On a tmpfs-``/tmp`` box, VACUUM spills a full DB copy — the builder pins
  its temp dir next to the output; if a "database or disk is full" appears
  anyway, ``df`` the **output path's** filesystem, not ``/``.
- The weekly workflow compares the database SHA-256 with
  ``data-artifacts.yaml`` and refuses mismatches.
