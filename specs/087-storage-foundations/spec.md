# Feature Specification: Storage Foundations — Compose Bootstrap + Storage Observability

**Feature Branch**: `feature/087-storage-foundations`
**Created**: 2026-07-03
**Status**: Draft
**Parent program**: Storage & Database Scaling Program (tri-county → Michigan → national res-7), Sprint 1 of 4–5. Approved plan: `~/.claude/plans/okay-now-if-you-gentle-falcon.md` (2026-07-03). Units S4 + S5 of the program's Tier-1 synthesis.

## Why (problem statement)

One canonical 520-tick michigan-canada run writes ~7 GB into `babylon_test`
(48,827 res-7 hexes × full row × every tick). Ten accumulated runs (71 GB)
exhausted `/var` and caused psycopg DiskFull failures (2026-07-03,
`project/08-graph-substrate.md`). Before the big levers land (delta
persistence — sprint 3/4; partition+archival — sprint 2), two foundations are
missing:

1. **Reproducible, correctly-provisioned database.** Today the test Postgres
   is a hand-rolled `docker run postgres:15` (spec-037 requires 16+ with
   PostGIS/pgvector/uuid-ossp), 64 MB shm, untuned defaults, anonymous volume
   on the constrained root disk (38 GB free — the DiskFull disk), and no
   compose file. `db:bootstrap` *silently skips* the PostGIS/pgvector parts of
   the canonical schema ("WARN missing extension").
2. **Storage observability.** Nothing records a run's storage footprint; the
   only probe is a live `pg_database_size` in `sim:status`. There is no gate
   that catches a storage regression, so the sprint-2/3 wins cannot be locked
   in (the way spec-069 locked in its read-count win).

## Owner decisions inherited from the program plan (2026-07-03)

- Trackability = **verifiable + replayable** (Constitution III.7); archives are
  **local only** — no cloud code paths.
- Design ceiling = **national res-7**; Michigan is the validation vehicle.
- M10 ruling respected: one Postgres; no columnar substrate, no federation.

## User Stories

- **US1 (contributor)**: On a machine with only git, docker, and mise, I run
  `mise run setup` and get a working toolchain, a healthy tuned Postgres 16
  container, and the canonical schema applied — then `mise run
  qa:e2e-regression` passes.
- **US2 (Percy/agent)**: After any headless run, the artifact bundle records
  what the run cost in storage (per-table rows + bytes, bytes/tick) so
  footprint changes are diffable across commits without a live DB.
- **US3 (CI/agent)**: `mise run qa:storage-budget` fails loudly when a change
  makes the simulation write materially more rows/tick than the committed
  baseline (the anti-regression gate for sprints 2–4).

## Functional Requirements

### S4 — Reproducible tuned database (compose)

- **FR-001**: A `docker-compose.yml` at repo root MUST define the isolated
  test Postgres service with `container_name: babylon-pg-isolated` and host
  port 5433, preserving every existing task/DSN/doc reference unchanged.
- **FR-002**: The image MUST be built from `docker/postgres/Dockerfile` based
  on pinned `postgis/postgis:16-3.4` (matching the integration-test
  containers) plus the `pgvector` extension package, satisfying spec-037's
  extension requirements (PostGIS, pgvector, uuid-ossp).
- **FR-003**: Extensions MUST be created in `template1` during first-boot init
  so `clean:testdb`'s DROP/CREATE DATABASE recycle keeps them without
  re-running init.
- **FR-004**: A repo-tracked `docker/postgres/postgresql.conf` MUST be mounted
  and applied (sized for a ≥16 GB host: shared_buffers, effective_cache_size,
  max_wal_size, checkpoint_completion_target, `wal_compression=zstd`,
  `pg_stat_statements` preloaded, `track_io_timing=on`).
- **FR-005**: `synchronous_commit=off` MUST be set for the `test` role only
  (ALTER ROLE), not cluster-wide. Safety argument: per-tick writes are
  idempotent (`ON CONFLICT DO NOTHING`, spec-056 monotonicity) and
  crash-resume replays deterministically from the last committed tick
  (Constitution III.7) — an async-commit tail loss is re-created bit-exact.
  **Superseded in part** (2026-08-11, ADR176 ruling 32 / P-F completion,
  issue #382): the "not cluster-wide" clause held only while the setting
  applied exclusively to the `test` role; the Director ruled it must also be
  DECLARED for the player, and no config audit could see a role-scoped
  `ALTER ROLE`. `docker/postgres/postgresql.conf` (today's player-equivalent
  surface — the game-managed player cluster is still rust-port future work)
  now also declares `synchronous_commit = off` cluster-wide; the `test`-role
  `ALTER ROLE` stays for CI parity (`postgresql.ci.conf` deliberately makes
  no declaration, leaving that cluster at PostgreSQL's default of ON).
- **FR-006**: The data volume MUST default to a named docker volume (portable)
  and honor `BABYLON_PG_DATA` (path ⇒ bind mount) via compose interpolation;
  Percy's `.env` points it at the 3.6 TB data drive so the DB leaves the
  constrained root disk.
- **FR-007**: `db:up`/`db:start`/`db:stop`/`db:down` MUST be rewired to
  compose with names preserved. `db:down` now *preserves* data (container
  removed, volume kept); a new `db:nuke` destroys the named volume too.
  `test:int-pg` MUST reuse `db:up` instead of its inline `docker run
  postgres:15`.
- **FR-008**: A `mise run setup` task MUST bootstrap a fresh clone: mise
  toolchain → poetry install → pre-commit install → compose up (healthy) →
  `db:bootstrap` → reference-DB presence check with actionable message.

### S5 — Storage observability + budget gate

- **FR-009**: The headless runner MUST emit a top-level `storage` block in
  `manifest.json` (pattern: spec-069's `bridge_db_reads`): DB total bytes,
  ticks persisted, and per-table `{total_bytes, session_rows,
  session_rows_per_tick}` for the per-tick table families. `summary.json` is
  NOT touched (it is the semantic baseline file; byte-noise would pollute
  `tests/baselines/*.json` on regeneration).
- **FR-010**: The block MUST be built by a pure builder (unit-testable,
  decoupled from Postgres — same pattern as `run_summary.build_summary`) fed
  by one catalog query; failure to collect storage stats MUST NOT fail the
  run (best-effort, logged).
- **FR-011**: `sim:status` MUST additionally print the top per-tick tables by
  total size with live rows and average bytes/tick.
- **FR-012**: `tools/storage_budget.py` MUST provide `generate` (write
  baseline from a bundle's manifest) and `check` (compare a bundle against
  `tests/baselines/storage-budget-5t.json`, default tolerance +10% on
  rows/tick per table; under-budget passes with a note; bytes are
  informational only — rows/tick is the deterministic signal).
- **FR-013**: `mise run qa:storage-budget` MUST be self-contained: run the
  5-tick detroit-tri-county strict scenario, then `check` its bundle.

## Success Criteria

- **SC-001**: Fresh volume → `mise run setup` → `qa:e2e-regression` passes;
  `db:bootstrap` completes with **zero** "WARN missing extension" lines.
- **SC-002**: `qa:e2e-regression` output is semantically identical pre/post
  container swap (compare-bundle gate green; `trace.csv` unaffected — the
  storage block lives only in `manifest.json`, outside `input_hash` and the
  4 gate fields).
- **SC-003**: A run bundle's `manifest.json` contains the `storage` block with
  non-zero `dynamic_hex_state` rows; `mise run qa:storage-budget` passes at
  baseline and fails when checked against a doctored baseline (verified in
  unit tests).
- **SC-004**: Postgres data files no longer live on the root disk on Percy's
  machine (`BABYLON_PG_DATA` honored).

## Out of scope (later sprints of the program)

Partitioning + local Parquet archival + `archival.py` implementation
(sprint 2, ADR pending); schema normalization v2 (sprint 2); delta
persistence with checkpoint frames + `tick_commit` hash chain (sprints 3–4);
`boundary_flow_register` treatment; national dress rehearsal (sprint 5).
