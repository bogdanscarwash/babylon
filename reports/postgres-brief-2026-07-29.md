# Postgres Optimization Brief — Babylon / Program 27 Phase 4

**Editor:** consolidated from six lane outputs (schema inventory, live forensics, CI, player-machine,
capabilities, graph-at-rest), re-verified against the live instance 2026-07-29.
**Instance of record:** PostgreSQL **16.14** (Debian 16.14-1.pgdg11+1), `babylon_test` on port 5433.
Extensions: `postgis 3.5.2`, `vector 0.8.5`, `uuid-ossp 1.1`, `pg_stat_statements 1.10`, `plpgsql`.
No AGE, no pgrouting, no h3-pg, no BRIN indexes, no materialized views, no unlogged tables.

---

## Editor's verification log

Every load-bearing claim below was re-run. Results, with the disposition of each lane's version.

| # | Claim | Verified result | Disposition |
|---|---|---|---|
| V1 | DB size composition | `total 5794 MB / immutable_reference_* 5271 MB / session partitions 388 MB` | **CONFIRMED.** 91% of the estate is reference data; all 78 sessions of simulation state are 388 MB (6.7%) |
| V2 | LODES per-session duplication | 73 distinct `session_id`, 20,350,225 rows, in **3 cohorts**: 3×2,190,817 + 1×575,866 + 69×191,332 | **CONFIRMED.** Cohort sums to 20,350,225 exactly |
| V3 | Hex delta sparsity | session `348f28fd…`: **10** distinct hex ticks vs **520** committed ticks | **CONFIRMED.** 510 of 520 ticks wrote zero hex rows |
| V4 | Duplicate indexes on partition parents | `boundary_flow_register` 7 defs/4 unique; `conservation_audit_log` 5/3; `dynamic_relationship_state` 3/2; `dynamic_consciousness_state` 3/2; `dynamic_hex_state` 1/1; `tick_commit` 1/1 | **CONFIRMED.** 7 byte-identical duplicate definitions; the two "index diet" tables are clean |
| V5 | `hex_spatial_map` session-invariance | **0** h3 cells disagree on county/state across all sessions; 45,572 distinct cells in 573,397 rows (**12.58×**) | **CONFIRMED** |
| V6 | Topology tables partitioned? | `dynamic_hex_state`=`p`, `tick_commit`=`p`; **`node_state`=`r`, `edge_state`=`r`, `hex_spatial_map`=`r`, `community_membership`=`r`** | **CONFIRMED.** Topology is NOT partitioned → session purge needs mass DELETE |
| V7 | `node_state` full-frame per tick | session `277eaecc…`: 17 ticks × 92 nodes = **1,564 rows exactly** | **CONFIRMED** |
| V8 | `node_state` JSONB share | heap 4,216 kB, JSONB 3,706 kB (**88%**), `max(pg_column_size)` = **1,846 B** | **CONFIRMED.** Nothing crosses the ~2 kB TOAST threshold → JSONB is stored inline, never compressed |
| V9 | Flake Postgres pin | `flake.nix:150` `pkgs.postgresql_17.withPackages`; `:198` `postgresql_16.lib` is libpq only | **CONFIRMED.** Player ships **17**, dev runs **16.14** |
| V10 | Apache AGE maturity | Graduated to Apache **top-level 2022-05-17** (entered incubation 2020-04-30) | **INVENTORY CLAIM FALSE.** AGE is not an incubator project; the graph lane's correction was right |

### Corrections that change recommendations

**C1 — the topology-write-share figure is an invalid composition. (invalidates a graph-lane headline)**
The graph lane computed topology at "~207 kB/tick = ~46% of the 452 kB/tick budget". I verified the two
halves come from **different sessions**:

- `348f28fd…` — 45,572 hexes, 520 ticks, 230 MB → **442 kB/tick**, and **`node_state` row count = 0**.
- `277eaecc…` — 92 nodes, 17 ticks — the session all topology measurements come from.

So topology writes are **additive to** the 442 kB/tick, not 46% of it. Consequence: **no session in the
estate has full campaign shape** (the hex-heavy session has no topology; the topology session has no
hexes and ran 17 ticks). The binding 452 kB/tick figure is a real measurement of one partial shape, not a
validated campaign projection. Any storage projection built on it — including this brief's — inherits
that caveat. **A single full-shape reference run is the cheapest way to retire this whole class of
uncertainty, and it should precede the Phase 4 sizing decisions.**

**C2 — "materialized views" splits into two different proposals; one is right, one is wrong.**
The capabilities lane rejected materialized views on staleness grounds (a `REFRESH` window cannot be
reconciled against the per-tick SHA-256 and the III.13 vault byte-gate). The schema lane proposed a
"baked tier" of per-tick county/national projections written *by the engine, inside the existing
envelope transaction*. These are not the same thing and only one carries staleness:

- **REJECT** `CREATE MATERIALIZED VIEW` + `REFRESH` — introduces a staleness window against two byte gates.
- **ADOPT** engine-written projection rows inside `persist_tick_atomic`'s transaction — no staleness
  window exists, because the projection commits atomically with the state it projects. The engine already
  holds these aggregates in memory; recomputing them in SQL from hexes is redundant work.

**C3 — the checkpoint bound must read `tick_commit`, not compute `tick/52`.**
Both read-path fixes are correct in shape, but the schema lane's `tick BETWEEN ($2/52)*52 AND $2` hardcodes
`CHECKPOINT_EVERY_TICKS` into SQL — and that same lane then asks (its Director question 7) whether the
cadence should be per-session metadata. Those two positions are inconsistent. The forensics lane's version
resolves it: bound by `(SELECT max(tick) FROM tick_commit WHERE session_id=$1 AND tick<=$2 AND is_checkpoint)`.
The cadence stays data, the query stays correct across a cadence change, and multi-resolution can vary
cadence per resolution without invalidating stored campaigns.

**C4 — `synchronous_commit=off` on the player: the two lanes disagree; `off` wins.**
The forensics lane wants `on` for the player ("persisted history could diverge from adjudicated history").
That objection does not survive the write path: `tick_commit` is written **inside the same transaction** as
the envelope, so a lost transaction loses its commit marker with it. The result is a *shorter* history, never
a divergent one, and resume-from-`tick_commit` + deterministic replay re-derives the lost tick bit-identically.
Per the PG docs, `synchronous_commit=off` carries **no consistency risk** (unlike `fsync=off`) and bounds loss
at ~3× `wal_writer_delay` (≤600 ms). **Adopt `off`, but as a declared durability ruling, not an inherited
default** — it is currently set invisibly via `ALTER ROLE test` (`pg_db_role_setting`), which no config audit
would catch.

**C5 — AGE's rejection stands, on different grounds than the inventory gave.**
The maturity objection is void (V10). Reject on: (i) openCypher is structurally dyadic — every relationship
has exactly one start and one end vertex, so a hyperedge needs the same Levi reification plain SQL already
provides, and generic Cypher traversal *invites* the pairwise decomposition Amendment D bans; (ii) it requires
`shared_preload_libraries = 'age'` plus a **server restart**, turning "start the game" into "edit
postgresql.conf and restart the cluster" on a player machine; (iii) it routes every tick's persistence through
an `agtype` translation layer. Grounds (i) and (ii) are decisive on their own.

---

## The two facts that should drive the whole design

**1. The estate is 91% duplicated reference data, not state.** 5,271 MB of 5,794 MB is
`immutable_reference_*`, carrying a `session_id` and re-inserted per campaign. All 78 sessions of actual
simulation state total 388 MB (~5 MB/session). The "76 sessions / 5.8 GB" framing is misleading:
evicting 75 of 76 sessions reclaims ~158 MB; deduplicating reference data reclaims **~4.4 GB — 28× more**.

**2. Reads dominate writes by ~16×.** From `pg_stat_statements` (lifetime-cumulative, `stats_reset` is NULL):

| query shape | calls | mean | total |
|---|---|---|---|
| `view_runtime_trace_emission` (entity + p_acquiescence/p_revolution) | 849 | **1,104 ms** | 938 s |
| `view_runtime_trace_emission` (entity + k) | 849 | **971 ms** | 825 s |
| `v_hex_state_asof` / county reconstruction | 71 | **2,521 ms** | 179 s |
| all simulation INSERTs (hex + boundary + relationship + consciousness) | 1.43 M | 0.06–0.10 ms | **111 s** |
| `immutable_reference_lodes_od_matrix` INSERT (reference load) | 25,650,402 | 0.04 ms | **960 s** |

The write path is cheap. The reference *loader* costs 9× the entire simulation write history, and the
as-of read path costs 16×. Optimization effort spent on tick-write throughput is misallocated.

---

# Profile 1 — CI

**Headline: there is no CI Postgres profile, and the Postgres suites may have zero PR coverage.**

`.github/actions/postgres-up/action.yml` runs `docker compose up -d --wait babylon-pg`, which mounts the
**dev-box** `docker/postgres/postgresql.conf` verbatim — a file whose own header says it is "Sized for a
>=16 GB dev host running the canonical headless simulation". `rg -n "postgresql.conf|shared_buffers|fsync"
.github/ .mise.toml` returns zero CI overrides. Against a `ubuntu-latest` runner (4 CPU/16 GB public,
**2 CPU/8 GB private**) that is actively harmful, not merely untuned:

- `effective_cache_size = 12GB` **exceeds total runner RAM** on the private profile. That is a planner lie
  that produces bad plans, not just slow ones.
- `shared_buffers = 4GB` = 50% of an 8 GB runner, plus `shm_size: 1g`.
- `max_wal_size = 8GB` on a 14 GB runner SSD that also holds the 5.7 GB reference DB and the postgis layers.

Compounding it, `postgres-integration` is gated `if: vars.CI_REFDB_READY == 'true'` **and** its test step is
`continue-on-error: true` (`main.yml:152,191`) — two independent ways for real assertions to be
non-blocking, and `main.yml`'s own comments record that both have already fired (the nonexistent
`tests/integration/observatory/` collection error).

**Order of operations matters here: un-gate before tuning.** Tuning a job nobody runs is worth nothing.
`requires_postgres` (2 test files) and `requires_reference_db` (39 files) are separate axes, so a tier
selecting `-m "requires_postgres and not requires_reference_db"` needs **no 5.7 GB download** and can run
ungated on every PR in the fast lane at ~2–3 min.

Then: fork `postgresql.ci.conf` (never edit the dev file). `fsync=off` + `full_page_writes=off` +
`wal_level=minimal` (+ `max_wal_senders=0`, or the server refuses to start) are sanctioned by the PG16 docs
for "temporary read-only clones where data can be easily recreated" — a CI cluster has no next boot.
Memory down to `shared_buffers=512MB` / `effective_cache_size=2GB`. **Keep autovacuum ON** with
`cost_delay=0`: turning it off also disables auto-ANALYZE, and against a 2,415-index schema that yields
plan-dependent flakiness reproducing on no developer's machine.

**Isolation: template clone + per-worker databases.** Transaction-rollback fixtures are structurally
incompatible here, for three independent reasons in existing code: `ensure_session_partitions` commits
**per family** deliberately (`partitioning.py:79-88` — a spanning transaction "deadlocks against concurrent
multi-table readers/purges, observed under pytest-xdist 2026-07-16"); production code under test uses a
`ConnectionPool` (rollback isolates one connection, a second pooled connection cannot see its uncommitted
rows); and `ensure_ddl_applied` requires `autocommit=True`. Per-worker databases make the 9 `ACCESS
EXCLUSIVE` parent locks **disjoint by construction** — cross-worker partition contention becomes impossible.
`shared_buffers` is per-cluster, so 4 databases do not quadruple memory. Use `STRATEGY WAL_LOG` (default) —
the template is schema-only (~15 MB), and `FILE_COPY`'s forced checkpoints are exactly the cost to avoid on
an `fsync=off` cluster. Seal the template with `ALLOW_CONNECTIONS false` **after** DDL, which makes PG16's
no-connected-sessions requirement unfailable rather than hopeful.

Per-test isolation inside a worker DB should reuse the shipped primitive, not invent one:
`drop_session_partitions` is documented as O(1) catalog work with "zero dead tuples and no VACUUM debt". That
also fixes the dev-box bloat directly — the 693 partitions measured (9 families × 77 sessions) are tests that
never dropped theirs.

**Determinism note:** `FROM postgis/postgis:16-3.5` is a **mutable tag** and the Dockerfile then runs
`apt-get upgrade -y`, so the image is non-reproducible by construction — beneath `qa-e2e-regression`, a
byte-identity gate. The repo pins sqlite to 3.53.1 with a lockstep constant and calls a bump a ceremony;
Postgres has no equivalent. Digest-pin the base and add `PINNED_POSTGRES_VERSION` with a guard test, mirroring
`test_python_version_pin_consistency`. `apt-get upgrade` remains a hole even then; closing it fully means
building Postgres in the flake, which is a larger lift and the eventual destination, not a now-item.

---

# Profile 2 — Dev box

Autovacuum is **not** a problem and should not be tuned: zero dead tuples system-wide on the hot tables,
`n_tup_ins` exactly equal to `n_live_tup` (no UPDATE/DELETE churn at all), and
`autovacuum_vacuum_insert_scale_factor` is already tuned to 0.05 for exactly this insert-only pattern. The
0.2 default the inventory flagged governs the update/delete path this workload does not use.

What is worth doing:

- **Move dev/CI to PG 17** (V9). The player ships 17; dev runs 16.14. Every config value, plan shape, and
  `pg_upgrade` rehearsal in this brief is currently unverified against the shipping major. Cheapest
  high-value fix in the document, and a precondition for validating the rest.
- **Drop the 7 duplicate indexes** (V4). Root cause found: `migrations/0026:81` uses
  `CREATE TABLE (LIKE … INCLUDING ALL)`, whose clones get auto-generated names; the original
  `CREATE INDEX IF NOT EXISTS ix_boundary_session_tick` then re-fires, fails to match the new name, and a
  second identical index is born. Every `(session_id, tick)` index is *additionally* a strict PK prefix.
  `dynamic_relationship_state` is 76% index by size with two identical 11 MB copies.
  `migrations/0027` already applied this reasoning ("index diet") to `dynamic_hex_state` — which is why V4
  shows it clean — but the other seven families never got it.
- **Add a redundant-index sentinel.** No test catches this; it survived from 0026 to today and
  `ensure_session_partitions` replicates it to every new session. A `pg_index` check (no two indexes with
  identical `indexdef` modulo name; no non-unique index whose column list is a PK prefix) belongs beside
  `check:vocabulary`.
- **Drop two dead LODES indexes** — `ix_lodes_od_year_home` (149 MB, 0 scans) and `ix_lodes_od_session_year`
  (134 MB, 18 scans, a strict PK prefix): **283 MB, zero behaviour change**.
- **Methodological warning on index drops:** `pg_stat_database.stats_reset` is NULL, so `idx_scan` is
  lifetime-cumulative — and *our own forensics perturbed it* (`ix_lodes_od_year_home` moved 0→3 during the
  audit). Snapshot `pg_stat_user_indexes` to a table and diff over a real workload window before dropping
  anything on statistics alone; cross-check against query sites in `src/`.

---

# Profile 3 — Player machine

**ADR104 already rules the strategy** (game-managed cluster, `initdb` into `~/.local/share/babylon/pg`,
unix socket only, superuser-in-own-cluster, built from the repo's own flake closure). The closure has
**landed** — `flake.nix:150-153` `postgresql_17.withPackages [postgis pgvector]` — so ADR104's
"not yet in the closure" note is stale. What remains unbuilt is exactly what `flake.nix:143-146` says:
the cluster-lifecycle code (initdb/pg_ctl/socket wiring, first-run DDL applier).

**initdb flags, each justified:**

```
initdb -D "$DATA_DIR/pg" -U babylon --auth-local=peer --auth-host=reject -E UTF8 --locale=C -k
```

`-k` (data checksums) is **not default in PG17** but PG18 flipped it to on-by-default — adopting early
aligns with upstream and costs nothing. On consumer hardware with no ECC and no RAID it is the *only*
silent-corruption detector, and it cannot be added later without downtime (`pg_checksums --enable` needs a
cleanly shut-down cluster and rewrites every block). `-E UTF8 --locale=C` explicitly, **not `--no-locale`**:
per the PG17 docs, `--no-locale` with the default libc provider yields **SQL_ASCII**, silently.

**Config template — the three dev values that must NOT be copied:** `listen_addresses='*'` (opens a TCP port;
ADR104 mandates socket only), `shared_buffers=4GB` (the 25%-of-RAM guidance in the PG docs is explicitly for
"dedicated database servers" — the player's box is not one), and `checkpoint_completion_target=0.9`
(already the PG17 default; an unjustifiable line invites cargo-culting).

Player values: `listen_addresses=''`, `max_connections=20`, `fsync=on`, `full_page_writes=on`,
`synchronous_commit=off` (C4), `wal_compression=zstd`, `max_wal_size=2GB`, `checkpoint_timeout=15min`,
`autovacuum_vacuum_insert_scale_factor=0.05`, `temp_file_limit=2GB`, `restart_after_crash=off`,
`random_page_cost=1.1`, `jit_above_cost` raised (the county plan compiles 29–47 LLVM functions on a
sub-second interactive read — pure loss), `logging_collector=on` + `log_destination='jsonlog'` into
`logs/postgres.log` at 10 MB rotation. `shared_preload_libraries` deliberately **empty** — `pg_stat_statements`
costs permanent shared memory for a diagnostic the player never reads; have the doctor command enable it and
restart when a report is actually being filed. `restart_after_crash=off` because the PG docs give this exact
case ("when PostgreSQL is being invoked by clusterware") — **the game is the clusterware**, and silent
self-restart would hide crashes from the Rust supervisor, violating III.11 Loud Failure.

**`shared_buffers` / `work_mem` / `effective_cache_size` / `jit` are the weakest claims in this brief.** The
reasoning is sound (rely on the OS page cache; leave RAM for the in-memory graph where adjudication actually
happens) but the specific numbers are engineering judgment with no benchmark for "game DB co-resident with a
simulation process". They need a dev-box measurement pass, not a citation.

**Config delivery:** append `include_if_exists 'babylon.conf'` to `postgresql.conf` once at initdb, and
regenerate only `babylon.conf` on updates. The game owns one file it may freely rewrite; the player's
`postgresql.conf` is never machine-edited. Mirrors the existing generated-`defines.yaml` discipline.

**Crash recovery is already a strong property and should be stated as one.** `tick_commit` is written last,
inside the envelope transaction, and resume reads the marker table rather than `MAX(tick)` — so a mid-tick
crash is **invisible by construction**; the player always sees the last fully-committed tick, never a
half-written one. Worst case with `synchronous_commit=off` is losing one in-flight tick (~452 kB),
re-simulated bit-identically.

**Two gaps that are real:**

1. **`rng_seed` is absent from `CampaignRecord`** (`babylon_meta.py:60-84` stores `campaign_id`, `slug`,
   `engine_version`, `defines_hash`, `last_tick`, `status`, timestamps — no seed). Determinism makes a
   campaign a pure function of `(rng_seed, ContentDigest, tick)`, so persisting two columns converts the
   worst failure mode from **"save lost" to "rebuild save"** — a progress bar. The cheapest recovery story
   in the system is currently unavailable.
2. **No disk-space awareness anywhere** — `rg -rln "disk_free|shutil.disk_usage|ENOSPC" src/babylon/` returns
   nothing. On a 30–80 hour campaign on a shared consumer disk, ENOSPC is a scheduled event, and today it
   surfaces as a Postgres PANIC with no player-actionable message. Budget ~10 GB free (campaign ~2.35 GB +
   WAL ceiling 2 GB + reference-in-PG ~0.6 GB post-dedup + the 4.2 GB sqlite source). Needs a preflight gate
   and a mid-run soft warning in the Ratatui client.

**Retention/export is already built — reuse it, don't port it.** `src/babylon/persistence/archival.py` has
`export_session_to_parquet`, `_verify_manifest_against_live`, `purge_session` (correctly fail-closed —
raises `ArchiveVerificationError` and deletes nothing on row-count drift), `query_archived_session` via
DuckDB `read_parquet`, and `upload_to_r2`. This is analysis tooling, not the tick path; per P27's division of
labour it stays **Python**, and the Rust engine needs no exporter. **One defect:**
`_verify_manifest_against_live` `continue`s on `to_regclass IS NULL`, so a table dropped between export and
purge **passes the gate whose entire job is proving the archive complete before destroying the original**.

Framing to state explicitly: the live save **is** the cluster directory, not a file. Parquet is the
retained-history/interchange tier; `pg_dump -Fc` is a support/pre-upgrade mechanism only, never the primary
save format.

**Major-version upgrades (17→18):** Babylon is unusually well-positioned — content-addressed flake closures
let it ship old and new server binaries transiently, a stronger story than GitLab Omnibus's version-tagged
apt packages. Prefer `--clone` (Linux 4.5+ Btrfs/XFS-reflink, macOS APFS — both v1.0 targets) over `--link`,
because `--clone` leaves the old cluster usable; **`--link`'s rollback window closes the moment the new
server starts**, and the UX must not offer "undo" after that point. Do not re-run `CREATE EXTENSION`
(schema definitions carry over); PostGIS needs `ALTER EXTENSION postgis UPDATE`. Because P0-2's seed makes
campaigns replayable, "export → fresh initdb → replay" is an excellent documented fallback when
`pg_upgrade --check` fails, though likely too slow to be the only path for a 100-year campaign.

---

# Profile 4 — Rust-engine persistence design (P27 Phase 4)

## D1. Fix the torn-tick defect first — it is a live save-corruption path

This is the one finding in the whole brief that is a **defect, not an optimization**. There are two
independent write paths in two separate transactions:

- `persist_tick_atomic` (`postgres_runtime/_spec_062.py:270-349`) writes 8 row families **+ `tick_commit`**
  in one transaction. It writes **no** `node_state`/`edge_state`.
- `persist_tick` (`postgres_runtime/_legacy.py:167-172`) opens **its own** `conn.transaction()` for
  `_persist_nodes` / `_persist_edges` / `_persist_graph_attrs`.
- `PerTickTransactionEnvelope` (`envelope.py:74-93`) has fields for hex/external/boundary/audit/
  consciousness/demographics/employment/relationship rows — **and no node or edge rows** — yet it is
  designated seam 10, "PORTED as the kernel replay unit" (`ai/bsl-architecture-standard.md:559`).

So **the adjudicating topology is the one thing not covered by the tick's commit marker.** A crash between
the two transactions leaves `node_state` rows for a tick whose envelope rolled back. And rehydration then
reads them:

```sql
-- src/babylon/persistence/postgres_runtime/_legacy.py:295
SELECT MAX(tick) AS max_tick FROM node_state WHERE session_id = %s
```

That is the documented anti-pattern (`CLAUDE.md`: "`MAX(tick)` ≠ last committed tick") applied to the graph,
and it can **rehydrate the engine from a torn tick**. Fix: add node/edge/hyperedge rows to the envelope,
delete the second transaction, and resolve the tick from `tick_commit`. Ship with a red-phase test
reproducing the torn tick.

## D2. Graph-at-rest verdict: plain relational (upgraded), not AGE, not a C extension

**Adopt versioned-relational topology with content-addressed hyperedges.** Nodes/edges/hyperedges as
ordinary partitioned tables; validity-interval time travel; field-scoped delta writes; hyperedges exposed
as whole objects through declared views.

Amendment D's own rulings dictate the at-rest shape — this did not need inventing:

- **S-10:** membership change is **whole-hyperedge replacement** (`remove` then `add` in one effect list);
  no `add-member`/`remove-member`, because "a partially-mutated hyperedge is unrepresentable".
- **D25:** a member list **is a set**; declared order is never observable.
- **S-22:** node/edge/hyperedge types are **closed registries** → `SMALLINT`, not `VARCHAR`.

The consequence is sharp and easy to get wrong: **a `(tick, hyperedge, member)` incidence table keyed for
independent mutation is the wrong shape** — it makes representable exactly the partial mutation S-10
declares unrepresentable. Instead, version the hyperedge and content-address its member set:

```sql
CREATE TABLE hyperedge_version (
    session_id UUID, tick INTEGER, hyperedge_id VARCHAR(64),
    hyperedge_type SMALLINT NOT NULL,        -- closed registry (S-22)
    member_digest  BYTEA   NOT NULL,         -- sha256 over ASCENDING member ids (D25)
    fields JSONB NOT NULL DEFAULT '{}'::jsonb,
    PRIMARY KEY (session_id, hyperedge_id, tick)
) PARTITION BY LIST (session_id);

CREATE TABLE hyperedge_member (
    session_id UUID, member_digest BYTEA, member_id VARCHAR(64),
    PRIMARY KEY (session_id, member_digest, member_id)
) PARTITION BY LIST (session_id);
```

Content-addressing means **an unchanged hyperedge costs zero member rows per tick** — the set is written
once and referenced by digest forever. Ascending canonicalization makes the digest stable, so D25 holds by
construction. Levi/incidence stays **internal storage** (S-8 permits this); the *exposed* read surface must
present one hyperedge as one row (`array_agg(member_id ORDER BY member_id)` in a `v_hyperedge_asof` view),
so no analyst can accidentally decompose a hyperedge into pairwise rows. That is also the fix for
`community_membership`, which today exposes raw incidence with no whole-hyperedge framing.

The **digest needs a specified byte layout, not an implied one** (CLAUDE.md: contracts must be
language-agnostic to the byte) — ascending member-id order plus a declared separator and encoding, written
down, or Rust and Python will compute different digests for the same set.

**Rejections.** AGE: see C5. Custom composite-type C extension: same per-platform packaging burden as AGE,
loses promoted-column indexability, and is the one option that tempts the algebra into the DB — amendment
territory, not a schema decision. pgrouting: orthogonal (a routing library over a user-supplied edge table);
narrowly interesting for hex-adjacency corridor queries someday, irrelevant to topology-at-rest.

**Unexploited and free:** `WITH RECURSIVE` has **zero uses** in `src/` — the multi-hop traversal capability
AGE is wanted for is unused, not unavailable. The analyst query "every SOLIDARITY edge that died in the year
before the rupture" is already expressible via `LEAD()` over `edge_state` (edge death = a validity interval
closing without a successor). Note `edge_type` is stored **lowercase** (`solidarity`, `tenancy`) against the
uppercase vocabulary in `CLAUDE.md` — normalize to the closed registry while the schema is being rebuilt.

## D3. Read path: bound now, materialize validity at the port

The as-of read is O(campaign) per single-tick read. `EXPLAIN` on `v_hex_state_asof WHERE session_id=… AND
tick=1000`: partition pruning works, but the `LEAD()` fill-forward forces
`Seq Scan (rows=455720) → Sort → WindowAgg` over **the entire session's hex history** before filtering to
one tick, because the `tick` predicate cannot push into the `intervals` CTE. `view_runtime_trace_emission`
is worse — a `change_ticks × intervals` range join with a **second** stacked window, whose own comment
concedes cost is `|change_ticks| × |hexes|`.

Measured 1,104 ms at **10** checkpoint frames. A 5,200-tick campaign has ~100. The linear term alone gives
~11 s; the join term extrapolates toward ~110 s per dossier read. **Nothing in the test estate catches this
because every fixture is short.** Also: the sort is 455,720 rows (~27 MB) today, inside `work_mem=64MB`; at
5,200 ticks it is ~4.56 M rows (~273 MB) — an external merge sort on every read, and on a default-configured
player instance (`work_mem=4MB`) it spills ~68× earlier than the dev box suggests.

- **now:** bound by the checkpoint (C3). Rows sorted drop 455,720 → 45,572 (**10×** today, **100×** at 5,200
  ticks), and growth in campaign length becomes **constant instead of linear**. Pair with
  `(session_id, h3_index, tick DESC)` — verified absent (`indexdef LIKE '%h3_index, tick%'` → 0), and it is
  exactly the `Sort Key: h.h3_index, h.tick` in the plan — which turns the bounded read into an index walk.
  The rewrite alone is the primary fix; the index alone still scans all history.
- **rust-port:** stop deriving validity and **store it** — `valid_range int4range` on the delta row,
  `USING GIST (session_id, valid_range)`, read as `valid_range @> $tick`. `btree_gist 1.7` is available and
  not installed; it is what lets `session_id` share the GiST index, and what would let
  `EXCLUDE USING gist` **prove** non-overlap of validity intervals rather than trusting the writer. Strictly
  better than a matview here: no staleness window to reconcile against two byte gates.

**Correctness precondition, and it must become a sentinel before the rewrite ships:** the checkpoint bound is
valid only if every checkpoint frame contains **every** hex. True today (45,572 rows on 10/10 checkpoints =
the full set), but undocumented and untested. If a future checkpoint is partial, the rewrite silently returns
an **incomplete frame** — a wrong-data bug, not a slow query.

**Also fix `view_runtime_trace_emission`'s tick predicate, and it may be a defect.**
`migrations/0023:59-75` reads `FROM dynamic_hex_state h` with **exact tick equality** and groups by `h.tick`
— against the sparse delta table, the precise anti-pattern `CLAUDE.md` warns about. Given V3 (10 distinct hex
ticks in 520 committed), `WHERE tick=$2` returns **zero rows for ~510 of 520 ticks**. The view predates the
as-of interface (0023 vs 0030) and was never migrated. Either the consumer carries forward (performance
item) or the trace has been silently sparse (defect touching `qa:regression` goldens and the 22-column trace
contract). **Unresolved from schema alone — Director/owner question.**

## D4. Write path: field-scoped delta, and COPY only where it pays

Row-level delta is **measured useless on nodes**: 1,472 of 1,472 comparable rows differ from the prior tick
(100.0%). But the churn is **2 keys out of ~40** — `dpd_state` (256 B) + `dependency_ratio` change on 81/81
nodes every tick, while `wealth` changes on **2 of 85** and `population` on **3 of 85**. With 110 distinct
keys and ~40 per node, **~80% of every node row is rewritten byte-identical**, and V8 shows the JSONB is 88%
of heap and never TOAST-compressed (max 1,846 B, under the ~2 kB threshold).

So: promote the volatile keys to their own narrow per-tick table (they *are* the real per-tick signal), leave
the rest in a delta-persisted attributes row that re-emits only when its digest changes, reusing
`select_hex_rows_for_emission` / `is_checkpoint_tick` verbatim. Edges get plain row-level delta — 45.6%
changed, so **54.4% of edge rows are pure duplicates**. Recovers ~153 kB/tick of topology write volume in
sessions that write topology (see C1 — this is additive to the 442 kB/tick, not a share of it).

**COPY is a narrow win, not a general one.** Measured in this instance: `INSERT INTO dynamic_hex_state` =
751,505 calls @ 0.06 ms ⇒ ~16,600 rows/s; `COPY _hex_state_tmp FROM STDIN` = 80 calls, 573,397 rows ⇒
~104,000 rows/s. **~6× per row** — but total simulation INSERT time is only 111 s against 1,762 s in two read
shapes. So: **use COPY for the checkpoint frame only** (45,572 rows: ~2.7 s → ~0.44 s; over 100 checkpoints,
~270 s → ~44 s — and the checkpoint tick is the one user-visible write stall). Leave delta ticks on psycopg3's
pipelined `executemany`. Where COPY must stay idempotent, the pattern is COPY into scratch then
`INSERT … SELECT … ON CONFLICT DO NOTHING` — which is exactly what `_hex_state_tmp` already does; reuse it.

**Idempotency is load-bearing, not theoretical:** 751,505 INSERT calls produced 410,148 rows — **~45% were
`ON CONFLICT` no-ops**, i.e. real replay traffic. Do not remove it casually.

**BRIN: downgraded to measure-later.** Three lanes disagreed; the sharpest analysis wins. BRIN **cannot be an
`ON CONFLICT` arbiter**, and every write uses `ON CONFLICT (<PK>) DO NOTHING`; `PARTITION BY LIST (session_id)`
+ a PK leading `(session_id, tick, …)` already delivers pruning-then-ordered-range-scan; and the partitions
are already small (100 MB max, 10 MB typical) *because* partitioning bounds them. The one genuinely
interesting variant — drop `dynamic_hex_state`'s composite PK, add `BRIN (tick)`, ~35 MB btree → ~65 kB — is
only sound if the atomic envelope becomes the **sole** idempotency mechanism, and given the 45% replay
traffic above that is a deliberate ruling, not a tuning tweak. **Measure at the port; do not adopt blind.**

## D5. Partitioning, retention, and the planner constraint

Keep `PARTITION BY LIST (session_id)` — it buys the O(1) purge and verified pruning. **Do not sub-partition
by tick**; the checkpoint bound gives the same locality at zero catalog cost.

**Add the topology tables** (V6: `node_state`, `edge_state`, `hex_spatial_map`, `community_membership` are all
unpartitioned `r`). Today, purging a session requires a mass `DELETE` of exactly the graph tables — precisely
the heap bloat and autovacuum stall `partitioning.py:15-18` was written to avoid.

**Retention is a planner requirement, not just a disk one.** `max_locks_per_transaction=64` against 2,964
partitions (9 families × 78 sessions, verified 693 for the session-partitioned subset): any query that fails
to prune to a constant `session_id` — a cross-session analytic, a bulk purge, a `pg_dump` of a save — locks
every partition it touches. That makes the standing "1 live session + parquet export" direction load-bearing
for **planning time**, and argues for enforcing it in code (auto-purge on session close) rather than trusting
convention. But calibrate the disk expectation: dropping all 77 sessions reclaims **388 MB**, not gigabytes.

**DEFAULT partitions are a latent hazard.** Each family has one, all empty today. Attaching a new LIST
partition while a DEFAULT exists makes PostgreSQL **scan the DEFAULT under `ACCESS EXCLUSIVE`**. Free while
empty; the moment a writer skips session init, every subsequent session creation pays a scan and serializes
against it — the same deadlock class as the 2026-07-16 xdist incident. Add a cheap emptiness assertion.

Also: **~31 DDL objects per "New Game"** (9 tables + 22 indexes). On a player machine that is a lot of DDL to
run for a menu click; with retention at one live session, per-session partitioning buys less than it costs.
Worth a ruling (see questions).

## D6. Types, and the Rust client

- **`h3_index`: TEXT(15) → `BIGINT`.** An H3 index **is** a u64; the 15-char form is its hex rendering.
  Text costs 16 B/row vs 8, widens the PK btree proportionally, and makes every comparison a string collation
  compare on **the exact column the as-of reconstruction sorts by**. `rg -n "h3" rust/ -g '*.rs'` finds no H3
  usage anywhere in the Rust tree, and `h3o` represents cells as a `u64` newtype — **the window to choose is
  open now**. Caveat: `migrations/0039` declares `CREATE DOMAIN h3index AS text CHECK (length=15)`, an ADR138
  contract — changing it is a domain-contract change, not a free edit.
- **Settle NUMERIC vs DOUBLE PRECISION — it is a determinism question, not style.** The lineages disagree
  (spec-062+ uses `double precision`, confirmed live on all nine `dynamic_hex_state` value columns; spec-037
  uses `NUMERIC` for `node_state.wealth`, `tick_summary.total_c/v/s`, `territory_snapshot` c/v/s). The Rust
  engine computes in `f64`; writing `f64` into `NUMERIC` round-trips through decimal and **can render
  differently than the frozen Python reference did** — and golden-vault renders read from the DB, gated by a
  *separate* byte-gate that has already drifted independently of every qa checkpoint. Recommend
  `double precision` for all value substance with a written tolerance policy; keep `NUMERIC` only where a
  value is definitionally decimal.
- **Domains: adopt or delete.** `0039` creates `probability`, `currency`, `ratio`, `labor_hours`, `fips5`,
  `fips2`, `h3index` and **no column is typed as one**. Declared-but-unadopted contracts rot; a domain is a
  base type plus a CHECK, so adoption is free at runtime.
- **Do NOT adopt native `ENUM`.** The estate's TEXT+CHECK history vindicates itself:
  `boundary_flow_register.flow_type/source_kind/dest_kind` were widened **three times** via guarded
  `ALTER … DROP/ADD CONSTRAINT` (0031, 0040). Native enums cannot drop or reorder values, and
  `ALTER TYPE … ADD VALUE` has transaction restrictions that fight idempotent-DDL-on-startup. If compactness
  is wanted, use `SMALLINT` + a lookup table.
- **The Rust engine likely needs no Postgres client at all.** `ai/bsl-architecture-standard.md:480-485` keeps
  Python as the glue for "database connections and persistence glue". Clean seam: **Rust emits
  `PerTickTransactionEnvelope`; Python persists it** — keeping `sqlx`/`tokio-postgres` + a TLS stack out of
  the engine's dependency graph and player packaging untouched. If a direct client is chosen instead, the P27
  design already names it: the **sync `postgres` crate** with a blocking pool (zero async in the tick path;
  sqlx rejected). Note there is **no `babylon-graph` crate yet** (`rust/crates/` has `babylon-kernel`,
  `babylon-bsl`, `babylon-md`, `babylon-tui`, `babylon-tui-python`) — the at-rest contract is genuinely
  greenfield and can be specified before the crate exists.

## D7. Privilege boundary — enforce Amendment AE by grants, not convention

Verified structurally clean today: `rg -rl "from babylon.persistence" src/babylon/engine/systems/` → **0
files** (none of the 34 systems import persistence), and all 10 `v_hex_state_asof` readers are under
`src/babylon/projection/**`. Postgres does real query work, and that work feeds **projection and observer
only**. **No constitutional escalation is warranted by any finding in this brief.**

But the boundary is currently protected only by the *absence of an import*. Make it enforced:

- Add to the `lint:imports` contract: **no system in the tick path may read from Postgres.**
- Use the existing precedents rather than inventing: `babylon_intel` (migration 0036) is a SELECT-only role on
  10 named views — the model for a Rust-engine-vs-projection split. Recommend the engine connect as a role
  with **no `SELECT` on projection views at all**, so "Postgres never adjudicates" is a grant, not a habit.
  `babylon_meta` (0037) is the schema-level precedent for client-owned epistemic state excluded from the tick
  hash. Extend it one schema further: `reference` (keyed by `ref_digest`, read-only) / `ledger` (MUTATE tier,
  engine sole writer) / `babylon_meta` (client only).
- **The `v_observer_*` seam specified in the P27 design does not exist** — 16 views exist, **zero** match
  `v_observer%`. Half the seam (`babylon_intel`) is built. Schedule it; don't rediscover it in Phase 4.

**One adjacent item genuinely touches the line, flagged not recommended.** If save/resume is ever implemented
by reading adjudicating state *back out* of Postgres — as `hydrate_graph` does today — then SQL evaluation
semantics (LEAD interval math, float round-trips, row ordering) become load-bearing for the engine's starting
state, colliding with S-2/S-14/S-15. Loading a save is legitimate; adjudicating from a **SQL-computed
reconstruction** is the boundary. The clean resolution is to hydrate only from **full-frame checkpoint ticks**,
which sidesteps interval math entirely. That should be ruled binding, not left incidental.

## D8. Reference tier: content-address it, don't just dedup it

The per-session copy is not an accident — it is a **determinism pin** (a campaign must be immune to a later
reference rebuild changing its inputs). That requirement is correct and must survive; it is being paid for
with the wrong key. Key reference rows by **`ref_digest`** (the reference build's sha256, already canonical at
`flake.nix:167` and contracted by ADR098), and record on the campaign row which digest it pins. Two campaigns
on the same build share one physical copy; a new build adds a second and old campaigns keep pointing at the
first. **Determinism pin preserved exactly; storage becomes one copy per reference build.**

Sizing, from V2's cohorts: collapsing 69→1 and 3→1 leaves 2,958,015 of 20,350,225 rows (**~85%** reclaimed,
~4.1 GB on LODES) — and the capabilities lane's content-level `DISTINCT` returned 2,190,817, implying the
small cohorts' rows are a **subset** of the large cohort's content, which would push the saving to ~89%.
**Confirm content identity, not just row counts, before collapsing** — this is a 20M-row re-key with a
data-loss failure mode if the cohorts are genuinely different geographies. Plus `hex_spatial_map` (V5): drop
`session_id`, 91 MB → ~7 MB, and it also removes the per-session sort of 47,515 spatial rows visible in the
county plan.

Also note the reverse-direction question: `CLAUDE.md` says "SQLite = reference data only (read-only build
product)", which reads as an argument that this data should not be in runtime Postgres **at all**. Confirming
that boundary decides whether this is a dedup or an eviction.

## D9. Archive / pgvector — a recall bug, not a performance item

`document_chunk` pairs a **default-parameter** HNSW index with a btree on `collection`, and pgvector applies
filters **after** the index scan. Per the docs, with default `hnsw.ef_search = 40`, a condition matching 10%
of rows yields **~4 rows on average**. So any Archive query shaped `WHERE collection = ? ORDER BY embedding
<=> ?` **silently under-returns** — invisible to any test asserting only "results are non-empty". Installed
`vector` is **0.8.5**, and iterative index scans landed in 0.8.0, so the fix is a GUC, not a schema change:
`SET hnsw.iterative_scan = strict_order` on the Archive/`babylon_intel` connection (prefer `strict_order` —
it preserves exact distance ordering, which matters if an Archive read is ever golden-pinned). This is better
than the per-collection partial indexes the inventory proposed: no N indexes to maintain. Also add the
**missing GIN index** on `document_chunk.metadata` — `pgvector_store.py:188` filters `metadata @> %s` and only
the HNSW + a btree on `collection` are declared.

Caveat: `document_chunk` has **5 rows** and `idx_scan=0`. There is no pgvector performance evidence in this
instance either way — **do not design Phase 4 Archive persistence off this DB.**

## D10. h3-pg — conditional, and later than it looks

Two corrections, one in each direction. **Weaker than claimed:** res5/res6 parents are **already
materialized** (`hex_cell.res6_parent`/`res5_parent`, populated by `h3.cell_to_parent`), so h3-pg's headline
coarse/fine clamping value is already served by denormalized columns. And **there is no hex tier in the map at
all** — `session.py:1099-1108` raises outside `(county, state, ea)` and `ea` returns `None` unconditionally
("honest absence"). So the MULTI-RESOLUTION NATIVE ruling currently has **no renderer path**, and that gap —
not SQL-side H3 algebra — is the binding constraint.

**Stronger than claimed:** packaging is nearly free —
`nix eval --raw nixpkgs#postgresql17Packages.h3-pg.version` → **4.2.3**, and the flake already ships Postgres
via `withPackages`, so adding `ps.h3-pg` is a one-line change, not a per-platform C build.

Where it would earn its place *once a hex tier exists*: `ST_AsText(h3_cell_to_boundary_geometry(cell))`
generates hex outlines server-side **in the exact WKT the existing Rust parser already consumes** — no client
change, no new parser, no geometry stored for hexes at all. Plus `h3_compact_cells`/`h3_uncompact_cells` for
the mixed-resolution frame, which have no materialized equivalent. **Determinism boundary if adopted:**
hierarchy ops are pure u64 bit-manipulation and agree across versions; the **geodesic** ops
(`latlng_to_cell`, `cell_to_boundary`) are floating-point and are **not** guaranteed identical between h3-pg
4.2.3 (C) and the Python pin (locked 4.5.0). Restrict SQL-side use to integer hierarchy ops and enforce it
with a sentinel, not a comment.

## D11. Two things that are NOT worth doing

**LISTEN/NOTIFY for engine→client push: there is no such seam.** The Rust client is an **in-process PyO3
extension** (`babylon-tui-python`, `crate-type = ["cdylib"]`, `#[pyfunction] fn run(py, host, config_json)`)
— it receives a Python `host` object and calls back through FFI. Tick-commit push is a **direct function
call**; adding a Postgres round-trip to notify a component sharing your address space is strictly worse. Add
NOTIFY's 8000-byte payload cap and at-most-once non-durable delivery and it is a clear no. *The seam that
will exist* at Phase 3/4 is different — Rust engine ↔ out-of-process Python observer — and for that,
polling `tick_commit` is simpler; if push is wanted, a **stateless** notify carrying only `session_id + tick`
(durable state staying in `tick_commit`) is defensible precisely because it cannot become an adjudicator.

**wal2json / logical decoding: escalation, not adoption.** It inverts the constitutional dependency by making
Postgres's WAL the source of engine change events, putting a persistence layer on the adjudication path. If
Phase 4 Host tiers want it, that is an **Amendment AE constitutional question for the Director** — flagged,
not recommended.

**PL/pgSQL for simulation logic** is correctly ruled out by the Constitution. The narrow legitimate use
(wrapping partition-maintenance DDL or the `hex_latest` two-phase UPSERT to cut round-trips) is real but
small, and the Rust port rewrites those call sites anyway. Skip.

---

# Change table

Each row names the table/query/screen it serves. `now` = against the frozen Python engine.
`rust-port` = P27 Phase 3/4. `post-v1.0` = after the Rust engine ships.

| # | Change | Profile | Serves | Impact | When |
|---|---|---|---|---|---|
| 1 | Un-gate the Postgres CI tier from `CI_REFDB_READY`; select `-m "requires_postgres and not requires_reference_db"`; drop `continue-on-error` | CI | `tests/integration/balkanization,persistence,engine` | Postgres estate goes from possibly-zero PR coverage and non-blocking to blocking every PR at ~2–3 min. **Worth more than every tuning knob combined** | now |
| 2 | Fix the torn-tick defect: node/edge rows into `PerTickTransactionEnvelope`, one transaction, resolve tick from `tick_commit` not `MAX(tick) FROM node_state` | Rust engine, dev | `node_state`/`edge_state`; `hydrate_graph` | Removes a live save-corruption path that can rehydrate the engine from a torn tick | now |
| 3 | Checkpoint-bound the as-of read via `tick_commit.is_checkpoint` (not `tick/52`) | player, dev, CI | `v_hex_state_asof`, `view_runtime_trace_emission`; every dossier/vault/observe page | Sorted rows 455,720 → 45,572 (10× now, 100× at 5,200 ticks); growth becomes constant, not linear. **Required before any 5,200-tick campaign ships** | now |
| 4 | Sentinel: every checkpoint frame contains every hex | dev, CI | the #3 rewrite's correctness precondition | Without it, a partial checkpoint makes #3 silently return incomplete frames — wrong data, not slow | now |
| 5 | Add index `(session_id, h3_index, tick DESC)`, ideally `INCLUDE` value columns | player, dev | `v_hex_state_asof` `Sort Key: h.h3_index, h.tick` | Turns the #3 bounded read into an index walk; verified absent today | now |
| 6 | Content-address the reference tier by `ref_digest`; drop `session_id` from `immutable_reference_*` and `hex_spatial_map` | player, CI, dev | `immutable_reference_lodes_od_matrix` (4852 MB), `_qcew_employment` (388 MB), `hex_spatial_map` (91 MB) | **~4.4 GB of the 5,794 MB estate**; ~960 s of per-session reference INSERT disappears; New Campaign becomes an O(1) metadata write. Preserves the determinism pin | now |
| 7 | Drop the 7 duplicate indexes + both PK-prefix `(session_id, tick)` indexes; fold the diet into `ensure_session_partitions` | dev, CI, player | `boundary_flow_register`, `conservation_audit_log`, `dynamic_relationship_state`, `dynamic_consciousness/demographics/employment_state` | ~21.7 MB of ~231 MB on the largest session; per-session relations 38 → 22 (−42%); fewer btree inserts on the two dense-every-tick writers | now |
| 8 | Sentinel: no duplicate `indexdef` modulo name; no non-unique index that is a PK prefix | dev, CI | prevents #7 regressing | This survived from migration 0026 to today with no test catching it | now |
| 9 | Drop `ix_lodes_od_year_home` (149 MB, 0 scans) + `ix_lodes_od_session_year` (134 MB, PK prefix) — after a `pg_stat_user_indexes` snapshot/diff | dev, CI, player | `immutable_reference_lodes_od_matrix` | 283 MB, zero behaviour change. Our own audit perturbed `idx_scan`, so snapshot first | now |
| 10 | `SET hnsw.iterative_scan = strict_order` on the Archive connection; add GIN on `document_chunk.metadata` | player, dev | `pgvector_store.py:186-192` narrator retrieval | Fixes a **silent recall bug** (default `ef_search=40` yields ~4 rows on a 10%-selective filter), invisible to non-empty assertions | now |
| 11 | Move dev/CI to PG 17 to match `flake.nix:150` | dev, CI | the whole estate | Removes a works-on-dev-fails-on-player class; **precondition for validating every config value here** | now |
| 12 | Fork `docker/postgres/postgresql.ci.conf` + `docker-compose.ci.yml`: `fsync=off`, `full_page_writes=off`, `wal_level=minimal` (+`max_wal_senders=0`), `shared_buffers=512MB`, `effective_cache_size=2GB`, `max_wal_size=2GB`, `listen_addresses='*'`. Keep autovacuum ON with `cost_delay=0` | CI | every Postgres CI job | Removes RAM/disk over-commit and a planner lie (`effective_cache_size=12GB` exceeds an 8 GB runner's total RAM) | now |
| 13 | CI isolation: sealed template DB (`ALLOW_CONNECTIONS false`, `STRATEGY WAL_LOG`) + per-xdist-worker clones; per-test cleanup via `drop_session_partitions` | CI, dev | `requires_postgres` suites; the 693 orphaned dev-box partitions | Makes the 9 `ACCESS EXCLUSIVE` parent locks disjoint by construction — the 2026-07-16 xdist deadlock class becomes impossible. Rollback fixtures are structurally incompatible (per-family commits, ConnectionPool, autocommit DDL) | now |
| 14 | Digest-pin `postgis/postgis:16-3.5` + pin the pgvector apt package + `PINNED_POSTGRES_VERSION` guard test | CI, dev | `qa-e2e-regression` byte-identity gate | Closes a determinism hole: a mutable tag + `apt-get upgrade -y` beneath a byte-identical gate | now |
| 15 | Fix `_verify_manifest_against_live`: absence of a table must be an error, not `continue` | player, dev | `archival.py:250` → `purge_session` | A table dropped between export and purge currently **passes** the gate that exists to prove the archive complete before destroying the original | now |
| 16 | Add `rng_seed` + canonical `ContentDigest` to `CampaignRecord` | player | `babylon_meta.campaign` | Converts the worst failure mode from "save lost" to "rebuild save". Costs two columns | now |
| 17 | Disk preflight (~10 GB) + mid-run soft warning in the Ratatui client; `temp_file_limit=2GB` | player | new-campaign flow; the client status line | Zero disk awareness exists in `src/babylon/` today; ENOSPC on a 30–80 h campaign currently surfaces as a PANIC | now |
| 18 | Resolve `view_runtime_trace_emission`'s exact-tick-equality read of the sparse delta table | dev, CI | `trace_emitter.py`, `qa:regression` goldens | Returns zero rows for ~510 of 520 ticks. Either a perf item or a silent-sparsity **defect** — unresolved from schema alone | now |
| 19 | Baked projection tier: engine writes per-tick county/national rows **inside** `persist_tick_atomic`'s transaction | Rust engine, player | dossier/watchlist/chronicle screens; replaces interactive reads of `view_runtime_trace_emission` | Kills the 1,104 ms interactive read with **no staleness window** (commits atomically with the state it projects). NOT a materialized view | rust-port |
| 20 | Materialize validity: `valid_range int4range` + `USING GIST (session_id, valid_range)` + `btree_gist`; optional `EXCLUDE USING gist` to prove non-overlap | Rust engine, player | as-of reads of hex/node/edge/hyperedge state | Replaces the `LEAD()` window with an index scan; converts a trusted writer invariant into a proved one | rust-port |
| 21 | Hyperedge at-rest: `hyperedge_version` + content-addressed `hyperedge_member`; `SMALLINT` types; whole-hyperedge `v_hyperedge_asof` view. Specify the digest byte layout | Rust engine | Amendment D hyperedges; fixes `community_membership`'s raw-incidence exposure | Unchanged hyperedge costs **zero** member rows/tick; S-10's "partially-mutated hyperedge unrepresentable" holds by construction | rust-port |
| 22 | Field-scoped delta: promote `dpd_state`/`dependency_ratio`/`mass_receptivity` to a narrow table; delta the rest by digest; row-delta the edges | Rust engine | `node_state`, `edge_state` | ~80% of every node row is currently rewritten byte-identical (2 volatile keys of ~40); 54.4% of edge rows are pure duplicates | rust-port |
| 23 | Partition `node_state`, `edge_state`, `hex_spatial_map`, `community_membership` by LIST(session_id) | player, dev | session purge / retention | Today purging a session needs a mass DELETE of exactly the graph tables — the bloat `partitioning.py` was written to avoid | rust-port |
| 24 | `h3_index` TEXT(15) → `BIGINT`; settle `double precision` for all value substance; adopt-or-delete the `0039` domains; keep TEXT+CHECK over native ENUM | Rust engine | `dynamic_hex_state` PK + every hex sort; vault-visible NUMERIC columns | ~36 MB/campaign + integer instead of collation compares on the exact as-of sort key. `rust/` has no H3 yet — **window open now**; but touches the ADR138 domain contract | rust-port |
| 25 | COPY for checkpoint frames only (via the existing `_hex_state_tmp` scratch pattern); leave delta ticks on pipelined `executemany` | Rust engine, dev | `dynamic_hex_state` checkpoint writes | ~270 s → ~44 s over 100 checkpoints; the checkpoint tick is the one user-visible write stall | rust-port |
| 26 | Enforce the AE boundary by grants + `lint:imports`: engine role with no SELECT on projection views; build the `v_observer_*` seam; three schemas (`reference`/`ledger`/`babylon_meta`) | Rust engine, player | the Amendment AE line | Today protected only by the absence of an import. `v_observer_*`: **0 of 16 views exist** | rust-port |
| 27 | Cluster lifecycle: `initdb -k --auth-local=peer --auth-host=reject -E UTF8 --locale=C`; `include_if_exists 'babylon.conf'`; player config template; `restart_after_crash=off` | player | first-run + every session | `-k` is the only silent-corruption detector on consumer hardware and cannot be added later without downtime. Avoid `--no-locale` (silently SQL_ASCII) | rust-port |
| 28 | Freeze-sentinel on shipped migration files (sha per file) | player, CI | the 41-file concatenated digest applier | Editing any historical migration silently re-applies all 41 on a player's machine — safe only while every migration stays idempotent, currently an unenforced convention | rust-port |
| 29 | Measure `shared_buffers`/`work_mem`/`effective_cache_size`/`jit` on a co-resident dev-box pass; then a full-shape reference campaign run | player, dev | the player config template; all storage projections | These 4 values are the weakest claims in this brief. And per C1 **no session has full campaign shape**, so the 452 kB/tick projection is unvalidated | rust-port |
| 30 | h3-pg (`ps.h3-pg` 4.2.3, one flake line): `ST_AsText(h3_cell_to_boundary_geometry(...))` + `h3_compact_cells`, hierarchy ops only | player | a hex tier in the Ratatui map pane | Conditional: **no hex tier exists** (`ea` returns None); res5/6 parents already materialized. Geodesic ops must stay off the SQL side (FP divergence vs the Python pin) | post-v1.0 |
| 31 | BRIN on `dynamic_hex_state (tick)` in place of the composite PK | player, dev | `dynamic_hex_state` checkpoint writes + tick-range scans | ~35 MB btree → ~65 kB, but makes the atomic envelope the **sole** idempotency mechanism against 45% measured replay traffic. **Measure, do not adopt blind** | post-v1.0 |
| 32 | `pg_upgrade` path: prefer `--clone` over `--link`, `--check` + disk gate first, `ALTER EXTENSION postgis UPDATE`, never re-run `CREATE EXTENSION` | player | 17→18 major upgrades | `--link`'s rollback window closes the instant the new server starts; the UX must not offer undo after that | post-v1.0 |
| 33 | Build Postgres in the flake (closes the `apt-get upgrade` reproducibility hole fully) | CI, dev, player | the byte-identity gates | The logical endpoint of the sqlite pinning discipline; substantially larger lift than #14 | post-v1.0 |
| 34 | Recursive-CTE analyst queries over `edge_state` (edge death = validity interval closing without successor) | dev, player | analyst/cartographer dossiers | Zero `WITH RECURSIVE` uses in `src/` today — the capability AGE is wanted for is unused, not unavailable | post-v1.0 |

## Rejected (with grounds)

| Option | Grounds |
|---|---|
| **Apache AGE** | openCypher is structurally dyadic → no hyperedge fidelity over plain SQL, and generic traversal invites the pairwise decomposition Amendment D bans; requires `shared_preload_libraries` + a **server restart** on a player machine; `agtype` translation on every tick write. *Not* rejected for maturity — it graduated to Apache top-level 2022-05-17 (the inventory's claim was false) |
| **Custom composite-type C extension** | Same per-platform packaging burden as AGE; loses promoted-column indexability; the one option that tempts the algebra into the DB (amendment territory) |
| **`CREATE MATERIALIZED VIEW` + `REFRESH`** | Staleness window cannot be reconciled against the per-tick SHA-256 **and** the III.13 vault byte-gate. #19 + #20 get the same speedup with no staleness semantics to defend |
| **LISTEN/NOTIFY for engine→client push** | No such seam exists — the client is an in-process PyO3 `cdylib`; push is a direct function call. Plus 8000-byte cap, at-most-once, non-durable |
| **wal2json / logical decoding** | Inverts the constitutional dependency (WAL becomes the source of engine change events). **Constitutional escalation if wanted, not an adoption** |
| **pgrouting** | Orthogonal — a routing library over a user-supplied edge table; no bearing on topology-at-rest |
| **PL/pgSQL for simulation logic** | Ruled out by the Constitution; the legitimate DDL-wrapping use is small and the port rewrites those call sites |
| **Tick-range sub-partitioning** | Multiplies an already-large catalog (2,964 partitions vs `max_locks_per_transaction=64`); #3 gives the same locality at zero catalog cost |
| **Transaction-rollback test fixtures** | Structurally incompatible: per-family commits in `ensure_session_partitions`, `ConnectionPool` visibility, `ensure_ddl_applied` needs autocommit |
| **`autovacuum = off` in CI** | Also disables auto-ANALYZE → plan-dependent flakiness against a 2,415-index schema. Use `cost_delay=0` instead |
| **UNLOGGED session partitions** | `PARTITION OF` does not inherit persistence, so reaching them means CI running a **different** `partitioning.py` path than production — the 2026-07-19 divergent-DDL incident class. Static-table sweep in the CI template only |
| **Autovacuum tuning on the dev box** | Zero dead tuples system-wide; `n_tup_ins` == `n_live_tup` (no UPDATE/DELETE churn); insert scale factor already at 0.05. Wrong place to spend effort |
| **`ST_AsGeoJSON()` for map geometry** | The Ratatui map has a working hand-rolled **WKT** parser with golden tests; `ST_AsText()` matches byte-for-byte. GeoJSON would require displacing a working parser |
| **Converting `immutable_reference_tiger_county.geometry_wkt` to PostGIS `geometry`** | The map reads county geometry from **SQLite**, not Postgres. Nothing spatially queries this table |

---

# Director-only questions

1. **Is the hex layer supposed to change between checkpoints?** Verified: 510 of 520 ticks wrote **zero** hex
   rows (10 distinct hex ticks vs 520 committed). Either 45,572 hexes are materially static across 10
   simulated years, or the delta write path is inert. Both readings are bad and imply opposite fixes, and the
   mechanism that must absorb H3-res-7 refinement currently has **zero production exercise**. This blocks the
   Phase 4 hex persistence design.
2. **Should `immutable_reference_*` (5,271 MB, 91% of the DB) live in runtime Postgres at all?** `CLAUDE.md`
   says "SQLite = reference data only (read-only build product)", which reads as a no; the runtime DB holds 73
   copies keyed by `session_id`. The answer decides eviction vs a `ref_digest` re-key — and I did not assume:
   the 3×2,190,817 and 1×575,866 cohorts may be larger geographies rather than copies, though a content-level
   DISTINCT suggests the small cohorts are **subsets**. Confirm content identity before any collapse.
3. **Is the `0039` `h3index` domain (text, length 15) a frozen ADR138 contract, or may the Rust schema store
   `h3_index` as `BIGINT`?** H3 is natively u64, `rust/` has no H3 code yet, and this is the exact column the
   as-of reconstruction sorts by — the window to choose is open now and closes for the life of the schema.
4. **Save/resume and the AE line:** must resume always replay from `tick_commit` + a **full-frame checkpoint**,
   or may the Rust engine rehydrate adjudicating state from a SQL-computed reconstruction? Checkpoint-only
   hydration sidesteps LEAD interval math and float round-trips entirely (S-2/S-14/S-15). I want this binding
   rather than incidental before Phase 4 designs the resume path.
5. **MULTI-RESOLUTION NATIVE — one table with mixed-resolution rows, or one per resolution?** This determines
   the PK shape, whether resolution belongs in the partition key, and whether checkpoint cadence can differ
   per resolution. Single biggest open input to the Phase 4 design. Related: what is the **res-7 refinement
   coverage budget** — storage per campaign swings from ~2.3 GB to 10 GB+ on that parameter, and it decides
   whether full-frame checkpointing survives at all. Also: when a region refines mid-campaign, is the new hex
   set a fresh checkpoint or deltas against an absent baseline?
6. **Durability contract:** accept `synchronous_commit=off` on the player — worst case losing the tick in
   progress (~600 ms, ~452 kB), re-simulated bit-identically, with no consistency risk per the PG docs? It is
   currently set **invisibly** via `ALTER ROLE test`, which no config audit would catch, so I want it declared
   either way.
7. **Is `view_runtime_trace_emission`'s sparse read a defect or a perf item?** It reads `dynamic_hex_state`
   with exact tick equality, returning zero rows for ~510 of 520 ticks. If the consumer does not carry forward,
   the 22-column trace contract has been emitting sparse rows for a long time — which touches `qa:regression`
   goldens and likely needs a §6.5 ceremony rather than a routine fix.
8. **Is "1 live session per player" enforceable in code (auto-purge on session close), or player-elective?**
   It turns out to be a **planner** requirement, not just disk: 2,964 partitions against
   `max_locks_per_transaction=64` means accumulated sessions degrade planning for everyone. Related: should a
   New Game be allowed to run ~31 DDL objects, or should the player tier collapse to `session_id`-keyed tables
   and give up drop-partition as the archive primitive?
9. **Is `CI_REFDB_READY` currently true?** If false, the entire real-Postgres CI surface has **zero PR
   coverage** today and change #1 is urgent rather than merely valuable. Also: is digest-pinning the postgis
   image a ceremony in the `nixpkgs-data`/sqlite sense? It is a determinism-contract change under a
   byte-identity gate, so I lean ceremony. And is moving dev/CI to PG 17 authorized now?
10. **Three items are defects rather than design questions** — the torn-tick envelope gap (#2), the purge
    verification skip (`archival.py:257`), and the missing `rng_seed` column. Do they land now as independent
    fixes against the frozen Python engine, or go on the Phase 4 slate? #2 in particular can corrupt a save
    today.

---

# Appendix — the six lane outputs, prettified

The lane outputs are preserved below with their own citations, risks, and questions, so the merged brief above
can be audited against its sources. Where a lane's claim was overturned, see the verification log (V1–V10) and
corrections (C1–C5) at the top of this document.

**Editorial note on this appendix:** each lane's *evidence* (findings, measurements, caveats, citations) and
its *own* risks and Director questions are preserved below. The lanes' long prose recommendation bodies are
**not** re-printed, because their substance has been merged into Profiles 1–4 above and duplicating them would
double this document without adding information. Where a lane's reasoning was preferred over another's, the
merge decision is recorded in C1–C5.

---

## Lane 1 — Schema inventory (as-built)

**Topic:** PostgreSQL runtime schema inventory — `src/babylon/persistence/`, feeding the Phase-4 Rust
persistence-layer design.

### Findings

- **Two independent DDL lineages that never FK-cross-reference each other.** (1) spec-037 bootstrap in
  `postgres_schema.py` — `POSTGRES_SCHEMA_DDL`, ~35 tables/views keyed by `game_session(id)` UUID PK with
  `ON DELETE CASCADE`. (2) spec-062+ "Cross-Scale Integration" applied via 32 numbered migrations
  (0010–0041) — `dynamic_hex_state`, `dynamic_external_node_state`, `boundary_flow_register`,
  `conservation_audit_log`, `dynamic_consciousness/demographics/employment/relationship_state`,
  `tick_commit`, `hex_spatial_map`, `runtime_political_*`, 10 `immutable_reference_*` tables. The second
  lineage uses a **bare `session_id UUID` with NO `REFERENCES game_session(id)`** — confirmed live:
  `pg_constraint` returned zero FK rows for any `dynamic_*`/`immutable_reference_*`/`boundary_flow_register`/
  `conservation_audit_log`/`tick_commit`/`runtime_*`/`hex_spatial_map` table.
- **PK strategy:** overwhelmingly composite natural keys, not surrogates, for per-tick state — near-universal
  shape `(session_id UUID, tick INTEGER, <entity_id>)`. `dynamic_relationship_state` and
  `boundary_flow_register` are 5-column. BIGSERIAL exceptions are the append-only audit/log tables
  (`game_turn`, `action_result`, `simulation_event`, two balkanization audits,
  `hex_r8_linear_features_reference`) where insertion order matters. `tick_event` uses
  `(game_id, tick, event_id SERIAL)`.
- **Column types:** value-substance fields are `DOUBLE PRECISION` with `CHECK(>=0)` in spec-062+ (migration
  0011); the **older** spec-037 lineage uses `NUMERIC` for money-like fields (`node_state.wealth`,
  `edge_state.value_flow`, `territory_snapshot` c/v/s, `hex_state.constant/variable_capital`,
  `tick_summary.total_c/v/s`) — a real float-vs-numeric split, not one convention. JSONB used pervasively for
  flexible payloads; **no plain JSON columns**. Enumerated strings are TEXT/VARCHAR + `CHECK(... IN (...))`
  **never native ENUM** — `boundary_flow_register.flow_type/source_kind/dest_kind` widened three times via
  `ALTER … DROP/ADD CONSTRAINT` (0031, 0040). Migration 0039 is the **only** `CREATE DOMAIN` usage
  (`probability`, `currency`, `ratio`, `labor_hours`, `fips5`, `fips2`, `h3index`) — declared, but no column
  in any DDL is actually typed `AS` one (ADR138).
- **Indexes:** `dynamic_hex_state` underwent a deliberate "index diet" (0027) — three secondary btrees from
  0011 dropped because `(session_id, tick)` is a strict PK prefix and county/state became NULL-heavy after
  normalization into `hex_spatial_map`. Live: only `dynamic_hex_state_pkey1` remains. GIST only for PostGIS
  geometry. HNSW exactly once (`idx_document_chunk_embedding`). **BRIN nowhere.** Partial indexes appear:
  `idx_node_state_org_type WHERE node_type='organization'`, `ix_hex_activity_hot`/`ix_hex_latest_hot WHERE
  heat>0`, `ix_audit_severity WHERE severity!='ok'`.
- **Partitioning:** 9 per-tick families are `PARTITION BY LIST (session_id)` — 8 converted in 0026, plus
  `tick_commit` born partitioned (0029). `ensure_session_partitions()` creates one partition per family per
  session, **one txn per family** to avoid deadlocks under pytest-xdist (documented 2026-07-16 incident);
  `drop_session_partitions()` is the O(1) purge primitive. A DEFAULT partition per family catches writers
  that skip session init.
- **Every FK is index-covered by a composite-PK prefix** — the live scan found 31 FK constraints, none
  lacking an index. `dynamic_relationship_state` additionally carries
  `ix_relationship_state_session_edge(session_id, edge_type, tension)` for the cross-tick
  `MAX(tension) FILTER (WHERE edge_type='EXPLOITATION')` aggregation (T080) — **the only index in the estate
  built for a cross-tick rather than per-tick scan**.
- **spec-089 sparse delta (`delta.py`):** measured **0 of 1,045** hex rows changed value across consecutive
  ticks in the canonical Michigan run (hex economics static-per-year by construction); full-frame-every-tick
  would be ~98% duplicate rows (7 GB measured, **~450 GB projected at national res-7**).
  `select_hex_rows_for_emission()` persists the full frame on checkpoint ticks (`tick % 52 == 0`) and
  otherwise only hex rows whose 9-field value tuple changed, tracked via an in-memory dict the bridge carries
  across ticks. Spatial keys NULLed for most rows (0027), living once per hex in `hex_spatial_map` with
  COALESCE-fallback reads. All 5 reconstruction views fill-forward via a `LEAD()` "intervals" CTE joined to a
  "spine" CTE (`tick_commit` UNION `dynamic_hex_state` ticks). `tick_commit` is what makes "last committed
  tick" well-defined once a tick can legally write zero hex rows.
- **Write path:** `PerTickTransactionEnvelope` is the unit persisted per tick — hex/external/boundary/audit/
  consciousness/demographics/employment/relationship rows, plus a replay-identity `determinism_hash`
  (`sha256(session:tick:seed)`), **distinct from** each `ConservationAuditRow`'s own content-hash
  `determinism_hash` — a documented naming collision (owner-queue item 31).
  `persist_tick_atomic` issues one INSERT per table with `ON CONFLICT (<PK>) DO NOTHING` for every family
  inside a single transaction — retry-after-crash-safe and idempotent by construction.
- **Conservation audit:** `conservation_audit_log` is append-only (`REVOKE UPDATE, DELETE FROM PUBLIC`, 0014),
  PK `(session_id, tick, scale, invariant_name)`; its `scale` CHECK widened twice via **guarded** ALTER (0031
  keys off `pg_get_constraintdef` so the ACCESS EXCLUSIVE ALTER fires only once per DB — an anti-deadlock
  guard for concurrent qa lanes). GATE-1 `determinism_hash` = `sha256(canonical-json(tick + sorted(hex_state
  by h3_index) + actions + rng_seed))`, graded ok/warn/alarm vs `GameDefines.economy.epsilon_conservation`
  and a fixed 1e-6 alarm threshold. **22 invariants enumerated in `_DEFAULT_INVARIANTS`, only 3 have concrete
  evaluators in this module** — the remaining ~19 are declared names with no evaluator wired here.
- **How vault/`observe()` reads hit the DB:** for LIVE play, `projection/*.py` view-builders read the
  **in-memory BabylonGraph, not Postgres** — the vault materializer bakes markdown from already-built View
  objects, no direct SQL. For HISTORICAL/replay, the bridge calls `RuntimeDatabase` (the SQLite backend is
  "honest-empty" — every `query_*` returns `[]`/`None`) or `PostgresRuntime`'s real implementations, which read
  `view_runtime_trace_emission` (the genuine `SUM(s)/(SUM(c)+SUM(v))` aggregate, contrasted in its own
  docstring against `territory_snapshot`'s single-hex-wins-on-conflict grain), the spec-037 snapshot tables,
  and `tick_summary` + `v_national_trend`.
- **Row-width hot spots at ~452 kB/tick:** `dynamic_hex_state` is the dominant per-tick **byte-size**
  contributor but **not** the dominant per-tick **row-count** contributor in steady state. Live: largest
  active partition 100 MB total/65 MB table (846,829 rows across sessions); `boundary_flow_register` second
  (65 MB/31 MB, 369,101 rows, **DENSE every tick** per spec-089 FR-007); `dynamic_relationship_state` third
  (45 MB/11 MB, 159,546 rows, **also dense every tick**, notably high total-vs-table ratio implying index/TOAST
  overhead on its 5-column key). Since delta ticks write near-zero hex rows, **`boundary_flow_register` and
  `dynamic_relationship_state` are the more likely steady-state contributors to the 452 kB/tick average.**
- **Privilege/schema-boundary precedents:** `babylon_intel` (0036) enforces Amendment V's observe-never-adjudicate
  at the **grant** layer — SELECT-only on 10 named views, INSERT+SELECT only on `document_chunk` and
  `narration_record`. `babylon_meta` (0037, separate namespace) holds CLIENT-owned epistemic state the engine
  never touches, explicitly excluded from tick-hash inputs — a **structural** implementation of the
  fog-epistemic-vs-material split. DDL idempotency uses a session-level advisory lock plus a digest-stamped
  `_babylon_schema_stamp` table so re-applying identical DDL is a pure-SELECT fast path (avoiding a documented
  xdist deadlock class: 46 setup failures + 7 deadlocks, 2026-07-16).

### Caveats declared by this lane

- 12,799 lines across `src/babylon/persistence/` under a constrained budget. Every migration and the
  small/medium modules were read in full; **not read**: `postgres_initialization.py` (1,310 lines);
  `postgres_runtime/_legacy.py` (2,984 lines — grepped + two sections spot-read);
  `hex_hydrator.py`/`sqlite_hydrator.py`/`tiger_ingestion.py` (skipped, unverified for hidden DDL/write paths).
- Did **not** confirm whether the ~19 evaluator-less conservation invariants are wired elsewhere in
  `src/babylon/engine/` — flagged unverified, not asserted absent.
- The 452 kB/tick figure was **supplied as binding context, not independently re-derived** byte-for-byte
  across every per-tick table in a single session.
- `protocols.py` (614 lines, the Protocol both backends implement) was not read directly — shape inferred
  from the two implementations.
- Live grants on `babylon_intel`/`narration_record` transcribed from migration SQL only; no live
  `pg_roles`/`information_schema` check.

### This lane's risks (verbatim substance)

Campaign-blocking O(session-history) as-of reads (1,104 ms / 2,521 ms measured at only 10 frames; super-linear
via the `change_ticks × intervals` join; **no test would catch it because all fixtures are short**); the sort
spill cliff (455,720 rows / ~27 MB today inside `work_mem=64MB` → ~4.56 M rows / ~273 MB at 5,200 ticks, and
~68× earlier on a player's default `work_mem=4MB`); 91% of the estate being duplicated reference data; the
`hex_spatial_map` 12.6× redundancy **compounded by it not being partitioned while being session-scoped**, so
purge needs a DELETE and defeats the very O(1) primitive it was session-scoped to support; exact duplicate
indexes on 7 of 9 families from `LIKE INCLUDING ALL` (0026:81) colliding with re-fired
`CREATE INDEX IF NOT EXISTS` under different names, with **no sentinel detecting it**; 2,964 partitions
against `max_locks_per_transaction=64`; the NUMERIC/DOUBLE PRECISION split drifting golden-vault renders
gated by a *separate* byte-gate; the `0039` h3index text domain fixing a representation before Rust has
chosen one; DEFAULT partitions forcing an ACCESS EXCLUSIVE scan on attach once non-empty; HNSW filtered search
silently under-returning.

### This lane's Director questions

NUMERIC vs DOUBLE PRECISION for vault-visible columns (declared drift ceremony, or preserve NUMERIC to keep the
byte-gate green?); is the `0039` `h3index` domain frozen; save/resume replay-vs-rehydrate; is "one live session"
enforceable in code (a **planner** requirement, not just disk); do `immutable_reference_*` carry `session_id`
for per-campaign modded overrides (flat dedup vs shared-base + override + COALESCE); MULTI-RESOLUTION — one
mixed-resolution table or one per resolution (**the single biggest open input to Phase 4**); should
`CHECKPOINT_EVERY_TICKS = 52` be a code constant or per-session DB metadata (a campaign written under one
cadence and read under another reconstructs **wrong frames**); does the game manage its own PostgreSQL instance
(several recommendations are configuration and only land if we control the config).

### Citations

`postgres_schema.py`; `migrations/0011,0013,0014,0019,0023,0024,0026-0031,0036-0041`; `delta.py`;
`envelope.py`; `hex_state.py`; `county_state.py`; `external_node.py`; `relationship_state.py`;
`audit_models.py`; `conservation_audit.py`; `partitioning.py`; `runtime_db.py`;
`postgres_runtime/_legacy.py` + `_spec_062.py`; live `mise run db:sql` on `babylon_test:5433`
(`pg_class` size, `pg_index`/`pg_am`, `pg_constraint` FK, row counts).

---

## Lane 2 — Live forensics

**Topic:** read-only forensics on the reachable runtime DB — version/config, size/index/dead-tuple audit,
`EXPLAIN` of the as-of read path.

### Findings

- **Reachable instance:** only the runner DB at `localhost:5433` responded via `mise run db:sql`; the 5432 web
  DB was **not exercised**. PG 16.14, `wal_level=replica`. `babylon_test` = 5,794 MB.
- **Settings:** `shared_buffers=4GB`, `work_mem=64MB`, `maintenance_work_mem=1GB`,
  `effective_cache_size=12GB`, `max_wal_size=8GB`/`min_wal_size=1GB`, `checkpoint_timeout=300s`,
  **`synchronous_commit=off`**, `jit=on`, `random_page_cost=4`, `effective_io_concurrency=1`,
  `max_connections=100`, autovacuum on with 3 workers. **`random_page_cost=4` + `effective_io_concurrency=1`
  are rotational-disk/cloud defaults** on a box whose `shared_buffers`/`effective_cache_size` **are** tuned.
- **Correction found by this lane:** `autovacuum_vacuum_insert_scale_factor = 0.05` **is already tuned**
  (configuration file), and `wal_compression = zstd` is tuned — the inventory's "default 0.2/0.1" framing was
  wrong. And **`synchronous_commit=off` is set by `ALTER ROLE test`** (`pg_db_role_setting`, role oid 10),
  **invisible in `postgresql.conf`**, applying to every database. `boot_val=on`, `reset_val=off`,
  `source=user`.
- **Top relations:** `immutable_reference_lodes_od_matrix` 4,852 MB (heap 1,807 MB — ~3 GB in indexes alone),
  `immutable_reference_qcew_employment` 388 MB, then the biggest `dynamic_hex_state` partition at 100 MB,
  `hex_spatial_map` 91 MB, `boundary_flow_register` partition 65 MB, `dynamic_relationship_state` 45 MB.
  **Reference tables dwarf per-session dynamic tables by ~an order of magnitude.**
- **Seq-scan hotspot:** the largest `dynamic_hex_state` partition shows `seq_scan=1301` with
  `seq_tup_read=268,464,738` against 455,720 live rows — i.e. **essentially every scan reads the entire
  partition** (~206k rows avg/scan).
- **`EXPLAIN` of the production query** (`HEX_FRAME_SQL`, `web/observatory/queries.py:83-92`, one
  session+tick, `ORDER BY h3_index LIMIT 51`) confirms the root cause: `Parallel Seq Scan` of the entire
  partition → `Sort` → merge join to `hex_spatial_map` → `WindowAgg` (LEAD per h3_index) over ~543k rows →
  `Nested Loop` against a 2-row `tick_commit` set → **only then** filtered to ~50 rows. The `spine` CTE's
  `tick=500` **does** push down to Index Only Scans; the cost is entirely the `LEAD()` window. `JIT: Functions:
  29`.
- **Decisive fact this lane established:** the table contains **only checkpoint frames**.
  `SELECT is_checkpoint, count(*), sum(hex_rows_written) FROM tick_commit` → `f | 510 ticks | 0 rows` and
  `t | 10 ticks | 455,720 rows` (45,572 each = the full hex set). `count(DISTINCT tick)` on
  `dynamic_hex_state` = 10.
- **`pg_stat_statements` corroborates** the plan concern: the `v_hex_state_asof`-shaped query 71 calls /
  179,026.9 ms / **2,521.51 ms mean**; the `p_acquiescence`/`p_revolution` entity query 849 calls /
  937,834.7 ms / 1,104.63 ms; the paired `view_runtime_trace_emission` query 849 calls / 824,649.1 ms /
  971.32 ms — **both look like a per-tick-per-entity read loop rather than a batched read.**
- **Composition of the 5,794 MB:** reference ~5,369 MB (**92.7%**), all 77 sessions of dynamic state 388 MB
  (6.7%), of which the one real 520-tick session is 230 MB (4.0%). Partition-bucket distribution: **3 large,
  93 small (<50k), 544 tiny (<1k), 71 empty** — the 76-session footprint is almost entirely abandoned short
  sessions (39 MB across 544 tiny partitions).
- **Independent confirmation of the write budget:** the hot session's partitions total 230 MB over 520 ticks =
  **442 kB/tick**, matching the briefed figure.
- **Unused indexes:** 1,227 with `idx_scan=0` totalling only **46 MB** — so the cost is **not disk**, it is
  1,227 indexes maintained on every INSERT for zero read benefit. `boundary_flow_register` is 400 unused of
  539 (**74% dead**) and is the top per-tick writer (301,539 rows / 519 ticks = **581 rows/tick**).
- **Dead-tuple picture is healthy:** no table exceeds 108 dead tuples system-wide; `n_tup_ins` exactly equals
  `n_live_tup` (no UPDATE/DELETE churn at all) on the hot tables; autoanalyze timestamps recent. **Autovacuum
  is the wrong place to spend Phase 4 effort.**
- **The Archive is empty** — `document_chunk` has 5 rows with an HNSW index at `idx_scan=0`. pgvector 0.8.5 is
  installed and unexercised; **there is no pgvector performance evidence in this instance either way.**

### Caveats declared by this lane

- The 5432 web DB was unreachable; `mise run db:sql` hardcodes 5433 (`.mise.toml:1179-1184`), so there is no
  configured path to the other instance. All findings are single-sided.
- **All `EXPLAIN`s are planner estimates** — no `EXPLAIN ANALYZE` (read-only constraint). Row estimates are
  visibly off (the as-of plan estimates 121,869 output rows where reality is ~47,515), so predicted speedups
  are **directional, not measured**.
- **`pg_stat_database.stats_reset` is NULL** — all counters are lifetime-cumulative since initdb across the
  whole 76-session history. The 960 s of LODES inserts and 1,302 seq scans are lifetime totals and **cannot be
  attributed to any single run or to current code.**
- **This lane's own forensics perturbed the dead-index evidence:** `ix_lodes_od_year_home` moved
  `idx_scan` 0 → 3 and `ix_lodes_od_session_year` 10 → 22 **during the audit**. Any index-drop decision based
  on live `idx_scan` is unsound; snapshot `pg_stat_user_indexes` to a table and diff over a real workload
  window first.
- No distinct chronicle/watchlist SQL query was found in `src/` to `EXPLAIN` — the hits were Rust TUI test
  files and a `chronicle_adapter.py` that builds summaries from in-memory event-bus payloads. A `watchlist`
  table exists (0 live rows) with no located SELECT.
- `random_page_cost`/`effective_io_concurrency` advice is **general best-practice inference**, not verified
  against this host's storage medium.
- Explicitly flagged **not** an escalation: read cost concentrating in Postgres is a projection/query design
  problem, squarely inside the sanctioned lane.

### This lane's Director questions

Is the hex layer supposed to change between checkpoints (**blocks the Phase 4 hex design**); should immutable
reference data live in runtime Postgres at all given `CLAUDE.md`'s SQLite ruling; is losing ~600 ms of
acknowledged commits acceptable for the player tier (`synchronous_commit=off` set invisibly via `ALTER ROLE`
on all databases); what is the **H3-res-7 refinement coverage budget** (storage swings ~2.3 GB → 10 GB+, and
it determines whether full-frame checkpointing survives); should a New Game be allowed to run DDL (~31 objects);
is the golden-trace path allowed to change shape (`view_runtime_trace_emission` gates `qa:regression` — likely
a §6.5 ceremony); should the 75 abandoned short sessions be exported to parquet or simply dropped (they look
like CI residue, and treating them as archive-worthy would build the export path against the wrong workload).

### Citations

`migrations/0030_views_current.sql:37-67`; `web/observatory/queries.py:52-100,83-92`; live `db:sql` battery
(`version()`, `pg_settings`, `pg_database_size`, `pg_extension`, `pg_total_relation_size`,
`pg_stat_user_tables`, `pg_stat_user_indexes`, `EXPLAIN` on `v_hex_state_asof`, `pg_stat_statements`,
`tick_commit` max-tick, `pg_db_role_setting`, `pg_roles`).

---

## Lane 3 — CI and the test estate

**Topic:** how CI uses Postgres — standup frequency, DDL application, per-test isolation, where CI time goes;
then the optimal CI Postgres profile.

### Findings

- **Two CI lanes.** `.github/workflows/ci.yml` (dev fast lane, push/PR to dev+main, ~8-10 min target) and
  `.github/workflows/main.yml` (full pipeline, main only). **Postgres is NEVER started in the fast lane** —
  all jobs use mocks or synthetic in-memory factories. `ci.yml`'s own comment: "the 5 regression scenarios are
  synthetic in-memory factories (no Django, no reference DB, no Postgres)."
- **Real Postgres only in `main.yml`'s `postgres-integration` and `qa-e2e-regression`** plus nightly's
  equivalents. Each is a **fresh ephemeral container per job run**, via the composite
  `.github/actions/postgres-up/action.yml`: `docker/bake-action` (buildx + GHA layer cache) then
  `docker compose up -d --wait babylon-pg`, from `docker/postgres/Dockerfile`. Deliberately **not** a GH
  Actions `services:` block, because a stock postgis image lacks pgvector and `init_schema` would fail at
  `CREATE EXTENSION vector` (comment, `main.yml:147-150`).
- **All four Postgres jobs are gated `if: vars.CI_REFDB_READY == 'true'`** (nightly's variant also ANDs on the
  cron string) — tied to owner item 40 Phase 6 (the 5.7 GB reference DB has no CI subset artifact).
  Comment: "A gated job is an EXPLICIT red, not a hidden one." **The live value could not be confirmed** from
  the sandbox (no gh/API access) — a real unknown, not a confirmed "never runs".
- **DDL is applied via ONE canonical idempotent path everywhere** — `ensure_ddl_applied(conn,
  POSTGRES_SCHEMA_DDL)` (`postgres_schema.py:116`). It stamps `_babylon_schema_stamp` with a SHA-256 digest of
  the applied chunk set and skips an already-stamped digest. **Confirmed live:** 5 stamp rows, digests dated
  2026-07-23 → 2026-07-27 — incremental migrations landing over time on a long-lived container, never re-run
  from scratch. `.mise.toml`'s `db:bootstrap` comment records a **2026-07-19 nightly incident** where a bare
  per-statement DDL loop silently diverged from production schema by omitting exactly the stamp-table DDL;
  `ensure_ddl_applied` was adopted to fail **loudly** instead (III.11).
- **The two-DB-port split is real and load-bearing:** 5432 = dev/"web" Postgres (Django alias), 5433 =
  `babylon-pg-isolated` (`babylon_test`, user/password test/test). `tests/conftest.py`'s `pg_dsn` defaults to
  the 5433 DSN unless `BABYLON_TEST_PG_DSN` overrides. **The same DSN string is repeated verbatim across at
  least 5 places** (workflows, mise tasks, conftest) — a de facto contract with no type enforcement.
- **The testcontainers per-test isolation pattern is now DEAD CODE.** `tests/integration/web/conftest.py`
  no longer exists as source (only stale `__pycache__` .pyc remain) — `main.yml:163-172` confirms it "was
  deleted (Director ruling, `test(web): remove the legacy Django test estate`)". The `django_db_setup`
  override in `tests/conftest.py` (lines 278-332) built to accommodate it is now dead weight, and the comment
  there is **inaccurate documentation** pointing at a non-existent file.
- **Non-web Postgres isolation is coarse:** ONE shared container per CI job (not per-test), schema applied
  once, then pytest `-m requires_postgres`, torn down with `docker compose down -v`. The step is
  **`continue-on-error: true`, labeled "advisory"** — its own comment states these suites "had NO
  Postgres-provisioned CI home" before this job, and a since-fixed collection error (nonexistent
  `tests/integration/observatory/` path, deleted with the Django estate) meant
  **"balkanization/persistence/engine never actually ran under this advisory label either"** until a
  2026-07-27 fix.
- **CI time DB-wise concentrates in `main.yml`'s heavy legs:** both Postgres jobs carry
  `timeout-minutes: 25`, each paying a from-scratch buildx image build + reference-DB fetch + DDL bootstrap
  before any test runs. The `test-rest` shard runs a 45-min budget but explicitly does **not** touch Postgres —
  it is slow due to xdist integration/property/scenario volume, not DB standup. **The fast lane pays zero
  DB-container cost by design.**
- **`requires_reference_db` (39 files) is a separate axis from `requires_postgres` (2 test files + the README).**
  The former gates on the 5.7 GB SQLite absence and is excluded from both `test:unit-ci` and `test:rest-ci`.
- **Local convenience tasks:** `db:up`/`db:start`, `db:stop`/`db:down`, `db:nuke`, `db:sql`, `clean:testdb`
  (drops+recreates+re-bootstraps, addressing exactly the test-tick accumulation bloat), `test:int-pg`.

### Corrections this lane made to its own inventory

| Inventory claim | Reality |
|---|---|
| base image `postgis/postgis:16-3.4` | **`16-3.5`** (`docker/postgres/Dockerfile:9`) |
| (not mentioned) | **A tuned `docker/postgres/postgresql.conf` already exists** and is bind-mounted, selected via `command: postgres -c config_file=…` |
| (not mentioned) | **`docker/postgres/initdb/01-babylon-init.sql`** already sets `ALTER ROLE test SET synchronous_commit = off` |
| "3 files match `requires_postgres`" | 2 test files + `tests/README.md` |

**And the headline the inventory missed:** there is **no CI Postgres profile at all** —
`rg -n "postgresql.conf|shared_buffers|fsync" .github/ .mise.toml` returns **zero** overrides, so CI runs the
dev-box config verbatim. Also: `template1` already carries postgis/vector/uuid-ossp courtesy of
`01-babylon-init.sql`, so a template clone inherits extensions free; and `01-babylon-init.sql:14-15` records a
paid-for lesson — "The entrypoint created `POSTGRES_DB` *before* this script and template1 changes don't apply
retroactively — equip it directly too."

### Caveats declared by this lane

- `vars.CI_REFDB_READY` truthiness **still unverified** (no gh/API access). If false, the entire real-Postgres
  CI surface is dark on every PR and the tuning work targets a job nobody runs. **The single load-bearing
  unknown.**
- Did not verify whether the `requires_postgres` test modules use internal savepoint/rollback fixtures (would
  need reading `tests/integration/{balkanization,persistence,engine}` directly).
- The two-DB-port convention is enforced only by repeated literal strings; no automated guard found.

### This lane's risks (verbatim substance)

`wal_level=minimal` **requires** `max_wal_senders=0` or the server refuses to start (piecemeal copying breaks
bring-up, and the failure looks like a mysterious hang behind `compose --wait`); overriding `config_file`
**discards the image's default `listen_addresses`**, so the CI conf MUST set it (otherwise `pg_isready` passes
on the unix socket while TCP is refused); `autovacuum=off` silently disables auto-ANALYZE → plan-dependent
flakiness against a 2,389-index schema; per-worker DBs multiply **connections** not `shared_buffers`, so
`max_connections=30` must cover 4 workers × pool 4 = 16 + bootstrap or it becomes a silent connection-refused
ceiling; `CREATE DATABASE … TEMPLATE` fails if any session is connected, and the `ALLOW_CONNECTIONS false`
seal creates an ordering trap for whoever debugs the template locally; UNLOGGED session partitions would
require CI running a **different `partitioning.py` path than production** — the 2026-07-19 divergence class;
`apt-get upgrade -y` keeps the image non-reproducible even after digest-pinning; disk headroom on the 14 GB
runner SSD is genuinely tight where the 5.7 GB reference DB coexists with PGDATA/WAL/postgis layers, so
`max_wal_size=2GB` is chosen to fit and raising it without redoing that arithmetic risks ENOSPC mid-job.
Explicitly noted: **R4 (digest pinning) actively DEFENDS** the per-tick SHA-256 contract; no escalation
required, flagged because the Phase 4 Host-tier design will be tempted to move adjudication into Postgres.

### This lane's Director questions

Is the repo public or private (4 CPU/16 GB vs **2 CPU/8 GB** runner — the sizing basis for the whole memory
block; sized for the worse case so it is safe either way); what is the live `CI_REFDB_READY` value; does moving
the Postgres tier into the fast lane (~2-3 min) fit the 8-10 min target and is a new blocking gate on dev PRs
acceptable; renaming `postgres-integration` would rotate a branch-protection required-check name
(`main.yml:176-178` says it was left stale for exactly this reason) — absorb the rotation now or add alongside;
is digest-pinning the postgis image a ceremony in the `nixpkgs-data`/sqlite sense (leaning ceremony, since it
is a determinism-contract change under the byte gate); should the `isolated_session` fixture be retrofitted
across existing suites or is that a separate cleanup train; **should this CI profile be designed against the
current schema or held until the Rust persistence schema settles** (designing twice is cheap; designing against
a schema about to be deleted is not); is building Postgres in the flake worth chartering for v1.0.

### Citations

`.github/workflows/{ci,main,nightly}.yml`; `.github/actions/postgres-up/action.yml`; `.mise.toml` (db:*,
test:* tasks); `postgres_schema.py:116`; `tests/conftest.py:130-446` (esp. 266-353); `docker-compose.yml`;
`docker/postgres/Dockerfile:9`; `docker/postgres/postgresql.conf`; `docker/postgres/initdb/01-babylon-init.sql`;
`pyproject.toml:263-270` (marker registration), `:66` (psycopg>=3.3.3); `uv.lock:4178` (xdist 3.8.0);
`partitioning.py:79-88,115-137`; live `_babylon_schema_stamp` (5 rows) and `information_schema.tables` (782)
queries; PG16 docs (non-durability, WAL config, async commit, CREATE DATABASE); GitHub runner reference.

---

## Lane 4 — Player machine

**Topic:** shipping PostgreSQL on the player's machine — bundling/initdb/first-run, per-campaign
database-or-schema, config template, save/export/retention, cross-version upgrade, failure-mode UX; and which
Rust PG client.

### Findings

- **ADR104 already rules the exact strategy** (`ai/decisions/ADR104_nix_bootstrap_installer.yaml`, amends
  ADR094, 2026-07-21): a **game-managed** cluster, `initdb` into `~/.local/share/babylon/pg`, **unix-domain
  socket only**, run as a child process of the game, **"superuser-in-own-cluster"** (so
  `CREATE EXTENSION postgis`/`pgvector` needs no host-admin step); built from the repo's own flake closure;
  reference DB shipped as a sha-pinned fixed-output derivation through the **same narinfo-signed R2 cache
  `install.sh` already trusts**. apt/PGDG bare-metal (`CONSTITUTION.md:674`) is a documented **fallback only,
  not primary/tested**.
- **The P27 design already names the Rust client and the pin**
  (`docs/superpowers/specs/2026-07-28-program-27-refoundation-design.md:370-374`): "Client: the sync
  `postgres` crate with a blocking pool — the scout verified **zero async anywhere in the current tick path**,
  so nothing forces tokio; **sqlx is rejected**." Diesel is not considered anywhere. `rusqlite` is the separate
  read-only reference-DB reader, and its sqlite 3.53.1 byte-identity pin binds only the **builder**, not
  read-time — so no read-time pin obligation for the PG client either (runtime dependency, not a determinism
  input).
- **The persistence contract commits to reproducing the modern envelope path verbatim** (same doc, 333-374):
  one transaction per tick, `ON CONFLICT DO NOTHING`, `tick_commit` written last in the same transaction;
  **resume reads the commit-marker table, never `MAX(tick)`**. The per-family transaction pattern (a paid-for
  deadlock lesson) and the 52-tick checkpoint cadence are **explicitly preserved rather than redesigned**.
  `v_hex_state_asof` stays a live Postgres VIEW queried from Rust — its LEAD/as-of semantics are **not**
  reimplemented in Rust, i.e. Postgres is doing real query work, consistent with PG maximalism. **The legacy
  full-snapshot Path A writer (`_legacy.py::persist_tick`) is flagged for DELETION, not porting** (Director
  sign-off already recorded) — directly answers the Phase-4 Path-A queue.
- **Real-world precedent** confirming ADR104's shape is industry-standard, not novel: the
  `postgresql_embedded`/`postgresql-archive` Rust crates download a matched server binary and run
  `initdb` + `postgres` under an ordinary user with unix-socket connectivity. **GitLab Omnibus** is the mature
  "ship a full Postgres inside my package" precedent — vendors Postgres under `/opt/gitlab/embedded`, and
  `gitlab-ctl pg-upgrade` wraps upstream `pg_upgrade` to migrate data files at package-upgrade time (symlink
  swap after shutdown + disk check). **Babylon's flake-pinned/Nix-closure delivery is a stronger
  reproducibility story** than GitLab's apt approach, since the exact PG+extension build is content-addressed
  and cache-verified rather than version-tagged.
- **Save/export:** the retention question is already answered by standing direction (1 live session + parquet
  export) plus the ADR098 parquet precedent. Generic `pg_dump -Fc`/`-Fd` (both compressed, parallel restore,
  selective/reordered restore) is the natural whole-save backup **if** the save IS the cluster; parquet is
  right for the retained-history tier (columnar, language-agnostic). **A live session's "save file" is really
  the Postgres data directory plus the checkpoint cadence — not a single flat file** — so `pg_dump` should be
  framed as EXPORT/BACKUP (sharing a save, or pre-`pg_upgrade`), never the primary persistence format.
- **Tuning direction from the official WAL docs:** `synchronous_commit=off` is the highest-leverage
  save-safety/speed tradeoff — a crash can lose recent allegedly-committed transactions (bounded by
  ~3× `wal_writer_delay`, sub-second) but **NEVER corrupts consistency**, unlike disabling `fsync`. Exactly the
  profile a single-player, single-writer, one-tick-one-transaction game wants: worst case replay the last tick,
  not touch integrity. `wal_compression` (lz4 fast / zstd better ratio, PG15+) is a good default given write
  volume with no replication bandwidth at stake. `shared_buffers` should be low relative to server defaults
  given modest hardware and co-residency — but **no game-specific numeric benchmark was found**, so any
  specific number is unverified and needs an empirical pass.
- **Crash-recovery UX is already designed, not a gap:** because `tick_commit` is written last in the same
  transaction and resume reads that marker, **a crash mid-tick is invisible to the resume path by
  construction**. Combined with the bounded loss window, the worst player-visible outcome is losing one
  in-flight tick (~452 kB), re-simulated deterministically. **State this explicitly in the Phase-4 Host-tier
  doc — it is a strong existing property.**

### Corrections this lane made to its own inventory

1. **The PG server closure HAS landed, and the pin is PostgreSQL 17** — `flake.nix:150-153`
   `pg-runtime = pkgs.postgresql_17.withPackages (ps: [ps.postgis ps.pgvector])`. ADR104's "not yet in the
   closure" note is **stale**. What remains unbuilt is exactly what `flake.nix:143-146` says: the
   cluster-lifecycle code (initdb/pg_ctl/socket wiring, first-run idempotent DDL applier + stamp table).
2. **Server major = 17**, not 16. `postgresql_16.lib` at `:198` is libpq-for-psycopg only, in the devshell.
3. **NEW: dev/player version skew** — dev is 16.14, player ships 17. Every 17-gated claim is currently
   untested locally. **The cheapest high-value fix in the document.**
4. **`immutable_reference_*` duplication measured:** 12 of the 13 tables carry `session_id` (only
   `immutable_reference_tiger_county` does not); `sqlite_hydrator.py:1-5` states the intent plainly —
   "SQLite → Postgres hydration for Spec 062 … **session-scoped** `immutable_reference_*` tables"; at ~250 B/row
   each full-scale campaign re-hydrates **~522 MB** of identical data for LODES alone.
5. **`v_observer_*` does not exist** — 16 views, **zero** matching `v_observer%`; the read-only role
   `babylon_intel` exists (0036), so half the seam is built. Also `test` is a **superuser** on the dev
   instance.
6. **`rng_seed` is absent from `CampaignRecord`** (`babylon_meta.py:60-84`).
7. **Zero disk-space awareness** — `rg -rln "disk_free|shutil.disk_usage|ENOSPC" src/babylon/` → nothing.
8. **`~/.local/share/babylon` measured at 34 MB** today (`logs`, `vault` — **no `pg/`**), confirming the
   lifecycle code is unbuilt. `install.sh:85-86` already establishes the XDG layout.
9. **The per-campaign layout verdict:** reject database-per-campaign (`CREATE DATABASE … TEMPLATE` is a
   filesystem copy, so each campaign clones the whole reference tier; extensions and `pg_upgrade` are
   per-cluster anyway — strictly worse) and reject schema-per-campaign (multiplies the DDL surface by N and
   breaks the single-apply model at `runner.py:302-321`, which concatenates all 41 migrations into **one**
   digest-stamped set; `v_hex_state_asof` would need per-schema instantiation and every query `search_path`
   juggling). **Adopt: keep LIST partitioning, separate schemas by TIER** (`reference` / `ledger` /
   `babylon_meta`), extending the `babylon_meta.py:1-16` precedent — "the engine never touches
   `babylon_meta.*` and no tick-hash input derives from it … the structural boundary". The 693 partitions are a
   **retention failure, not a design failure**: with 1-live-session enforced, steady state is 9.
10. **Migration mechanism:** `runner.py:302-321` globs `[0-9]*.sql` (not `00*.sql`, so numbering past 0099 does
    not silently drop files — another paid-for lesson) and hands the whole set to `ensure_ddl_applied` as one
    digest. Two consequences: every migration must remain idempotent **forever**, and **editing any historical
    migration changes the set digest and re-applies all 41** on a player's machine.
11. **`archival.py` already implements the whole retention flow** — `export_session_to_parquet:172`,
    `_verify_manifest_against_live:250`, `purge_session:272` (fail-closed, raises `ArchiveVerificationError`,
    then `drop_session_partitions` + leftover sweep), `query_archived_session:343` (DuckDB `read_parquet`),
    `upload_to_r2:385`, `_sha256_file:151`. **One defect:** `_verify_manifest_against_live` `continue`s on
    `to_regclass IS NULL`, so a dropped table **passes** the completeness gate.
12. **AE boundary verified structurally:** `rg -rl "from babylon.persistence" src/babylon/engine/systems/` → 0
    files; all 10 `v_hex_state_asof` readers are in `src/babylon/projection/**`; the only engine-side references
    (`headless_runner/runner.py`) are `COUNT(*)` preflight checks, not adjudication inputs. **No escalation
    warranted.** But the boundary is protected only by the absence of an import → make it a `lint:imports`
    contract. **Consequence to design around now:** the BAKED tier needs no DB (vault files + git), so the
    lobby *should* work with Postgres down — but the campaign catalog lives in `babylon_meta` **in Postgres**,
    so today it does not.

### Caveats declared by this lane

- The four memory values (`shared_buffers`/`work_mem`/`effective_cache_size`/`jit`) are engineering judgment;
  the PG docs' 25%-of-RAM figure is explicitly for **dedicated** servers and no benchmark exists for a
  game-co-resident DB. `effective_cache_size` allocates nothing (planner hint only), so it is the safest of
  the four.
- `pg_upgrade --link` makes the old cluster unusable **the moment the new server starts**, with no undo; before
  first start it is recoverable only by renaming `global/pg_control`'s `.old` suffix — a step no player will
  perform unaided.
- De-duplicating the reference tier **touches the determinism pin**: without read-only grants and
  content-address verification, two campaigns could share a mutated reference row and diverge **silently**,
  since reference data is an adjudication INPUT.

### This lane's Director questions

Is reference de-duplication a Phase 4 unit, or does re-keying a determinism pin need its own ADR + rider?
**Can the 4.2 GB reference sqlite be DELETED after hydration** (~10 GB preflight vs ~5.5 GB install, traded
against offline capability)? Should the lobby/campaign list work with Postgres **down** (a real robustness win,
but only if decided before the Host tiers are built)? Confirm `synchronous_commit=off` as the shipped
durability contract? `pg_upgrade` or export+replay for major upgrades (replay likely too slow as the only path
— documented fallback when `--check` fails)? Is moving dev/CI to PG 17 authorized now? Does a fourth log sink
(`logs/postgres.log`, jsonlog, 10 MB rotation) fit the 2026-07-28 logging directive or should Postgres log
through the game's own sink? Do the three found defects (purge verification skip, missing migration-freeze
sentinel, absent `rng_seed`) land now as independent fixes or on the Phase 4 slate?

### Citations

`docs/superpowers/specs/2026-07-28-program-27-refoundation-design.md:333-390`;
`ai/decisions/ADR104_nix_bootstrap_installer.yaml`; `CONSTITUTION.md:674`; `flake.nix:143-146,150-153,167,198`;
`install.sh:85-86`; `rust/Cargo.toml`; `rust/crates/babylon-kernel/Cargo.toml`;
`babylon_meta.py:1-16,60-84`; `sqlite_hydrator.py:1-5`; `archival.py:151,172,250,257-259,272,343,385`;
`engine/headless_runner/runner.py:302-321`; `postgres_runtime/_spec_062.py:146`;
`docker/postgres/initdb/01-babylon-init.sql:14-15,26`; `docker/postgres/postgresql.conf:30`;
`migrations/0018,0028,0036,0037`; PG17 docs (initdb, pg_upgrade, pg_checksums, WAL config, resource config,
pg_dump) + PG18 initdb; `github.com/theseus-rs/postgresql-embedded`; GitLab Omnibus PG-version docs;
`postgresqlco.nf` synchronous_commit.

---

## Lane 5 — Unexploited capabilities

**Topic:** Postgres capabilities Babylon is not yet exploiting vs. what is already in place.

### Findings

- **Extension inventory:** PG 16.14 with postgis 3.5.2, vector 0.8.5, uuid-ossp, pg_stat_statements, plpgsql,
  declared at `postgres_schema.py:167-169`. **No h3-pg, no wal2json, no pg_cron, no pg_partman, no AGE, no
  pgrouting.** Adding any is a net-new install decision, not a switch between deployed options.
- **pgvector is already used well** — `pgvector_store.py` implements HNSW + cosine (`embedding <=> query::vector`)
  for the Archive (spec 061), and HNSW is the currently-recommended index type (favored over IVFFlat for
  build-once/query-many, no training step). **No IVFFlat usage, so nothing to migrate off.** Not exploited:
  metadata GIN, and `hnsw.iterative_scan`.
- **Declarative partitioning is in real use, not a gap** — 9 families, per-session create + O(1) `DROP TABLE`
  purge instead of mass DELETE+VACUUM. **Directly matches the standing retention direction, already wired.**
- **PostGIS is used but inconsistently** — `hex_cell.geometry geometry(Polygon,4326)` / `centroid
  geometry(Point,4326)` with GIST (`postgres_schema.py:420-421,594`), same for `hex_map` (`:642,1144`). But
  `src/babylon/reference/schema.py:126` (`dim_county_geometry.geometry_wkt: Text`) and `:1732` store boundaries
  as **WKT-in-text** — no spatial index, no `ST_Intersects`/`ST_DWithin` queryability. *(Editor: this is the
  SQLite reference schema, and the map reads county geometry from SQLite — see lane-5 correction 0.1.)*
- **H3 has NO SQL-side presence at all** — the engine calls the Python `h3` library directly
  (`domain/geography/h3_mesh.py`: `h3.grid_disk`, `h3.cell_to_latlng`) and Postgres stores H3 indexes as
  opaque VARCHAR(15). h3-pg (PG13-17, matching 16.14) would let `cell_to_parent`/`cell_to_children`/
  `compact_cells`/`uncompact_cells` run as SET-based SQL. **This is a PROJECTION-layer capability, not
  adjudication — it does not cross the AE boundary.**
- **Composition views are ALL plain views** — `v_hex_economic`, `v_hex_mobilize`, `v_hex_aid`, `v_hex_heat`,
  `v_hex_intel` (`postgres_schema.py:1084-1123`) and `v_hex_state_asof`. **Zero `CREATE MATERIALIZED VIEW`
  anywhere in `src/`** (verified: `rg 'MATERIALIZED' src/` → 0 hits; live `relkind='m'` → 0 rows). Adoption
  would need an explicit refresh contract keyed to `tick_commit` and introduces a staleness window against
  III.13.
- **LISTEN/NOTIFY and logical decoding entirely unused** — no `pg_notify`/`LISTEN`/`NOTIFY`/wal2json anywhere
  (only unrelated in-process "notify observers" callback naming in `engine/observer_adapter.py`). NOTIFY's
  **8000-byte payload cap** and **at-most-once, non-durable** delivery (a client not listening at NOTIFY time
  never gets it) make it a poor fit for durable event delivery; defensible only as a lightweight
  "something changed, re-poll" wakeup.
- **No BRIN, no generated columns, no range types + exclusion constraints, no composite types/domains in use,
  no table inheritance outside partitioning, no UNLOGGED tables** — except `trace_log`, documented UNLOGGED at
  `postgres_schema.py:12`. *(Corrected by this lane: that is a **docstring only** — live
  `relpersistence <> 'p'` → **0 rows**; the table does not exist. All 757 public tables are `'p'`.)*
- **Server-side PL/pgSQL essentially unused for logic** — consistent with the Constitution. The narrow
  legitimate use is the persistence/projection tier (wrapping the `hex_latest` two-phase UPSERT or
  partition-maintenance DDL to cut round-trips); a stored procedure computing simulation state would not be
  legitimate, and none do.
- **Storage measurement (this lane's headline):** `pg_database_size` 5,794 MB; `immutable_reference_*`
  **5,271 MB (91%)**; all `dynamic_*`/`boundary_*`/`conservation_*` **387 MB**.
  `immutable_reference_lodes_od_matrix` = 4,852 MB = **84% of the entire DB**, with `session_id` **first in the
  PK** (`migrations/0016:9,16`) and a **2,761 MB unique index** showing 25.6 M scans returning 0 tuples (the
  insert-time uniqueness-check signature). Content-level `DISTINCT` over
  `(year, home_hex, workplace_dest, workplace_dest_kind, s000_workers)` → **2,190,817 unique of 20,350,225
  stored (9.3×)**. `immutable_reference_qcew_employment` shows the same shape (77 sessions, 1,976,941 rows,
  388 MB). **Together 5,240 MB = 99.4% of all reference data.**
- **Loader is row-by-row:** `lodes_commute_matrix.py:385` uses `cur.executemany(...)` — 25.6 M
  individually-planned statements, **960 s measured**. psycopg is pinned **3.3.4** (`uv.lock:3650-3651`), which
  supports `cursor.copy()`; measured COPY in this instance is ~104,000 rows/s vs ~16,600 for INSERT (**~6×**).
  Because the statement carries `ON CONFLICT DO NOTHING`, the correct shape is COPY into an UNLOGGED staging
  table then one `INSERT … SELECT … ON CONFLICT DO NOTHING`.
- **Dead LODES indexes:** `ix_lodes_od_year_home` 149 MB **0 scans**; `ix_lodes_od_session_year` 134 MB, 18
  scans, and a **strict leading prefix of the PK** so the PK index serves every query it could. **283 MB, zero
  behaviour change.**
- **The as-of read is worse than linear:** `EXPLAIN` of `count(*) FROM v_hex_state_asof WHERE session_id=…` →
  `Nested Loop (cost=69033..5825925, rows=36,822,176)` with
  `Join Filter: ((h.tick <= tick_commit.tick) AND ((lead(h.tick) OVER (?)) IS NULL OR tick_commit.tick <
  (lead(h.tick) OVER (?))))`, `WindowAgg (rows=455,720)`, `Seq Scan`, `JIT: Functions: 28`. **36.8 M estimated
  rows for a session at tick 468** (45,572 hexes × 10 frames vs 520 committed ticks) — a nested loop, so cost
  grows with the **product**. Fix: materialize validity as `int4range` + `USING GIST`; **`btree_gist 1.7` is
  available and not installed.**
- **`view_runtime_trace_emission` is the measured per-tick hot spot** — 849 calls / 937,835 ms / **1,104.63 ms
  mean** plus 849 / 824,649 ms / 971.32 ms = **~2.08 s of server time per tick**, extrapolating to **~3 hours**
  of pure projection query time in a 5,200-tick run. Honest caveat: `pg_stat_statements` means mix session
  sizes (a cheaper 258-call variant averages 36 ms), but the 45,572-hex session is the campaign shape.
  **Correctness concern:** `migrations/0023:59-75` reads `FROM dynamic_hex_state h`, joins on `cs.tick = h.tick`
  and groups by `h.tick` — **exact tick equality against the sparse delta table**, the documented
  anti-pattern; the session has `maxtick=468` but only **10 distinct ticks**, so `WHERE tick=$2` returns
  **zero rows for ~458 of 468 ticks**. The view predates the as-of interface (0023 vs 0030) and was never
  migrated.
- **Index census:** 2,415 btree, 2 gist, 1 hnsw, **0 BRIN**. `dynamic_hex_state` partitions carry **only** a
  pkey (largest 35 MB, 503,243 scans). Partitions are already small because LIST partitioning bounds them, so
  BRIN's honest impact is **small**. Live partition count **693 = 9 × 77 sessions**, exactly.
- **The Ratatui map's wire format is WKT, not GeoJSON** — `rust/crates/babylon-tui/src/views/map.rs:217-256`
  (`wkt_exterior_rings()`) accepts only `POLYGON`/`MULTIPOLYGON` and **loudly rejects** everything else (tests
  at `:832-833` assert `LINESTRING` and degenerate rings return `None`; `:865-867` makes malformed WKT a LOUD
  protocol failure); the wire field is `wkt: Option<String>` (`:112-113`), written by `session.py:1129`. So
  **`ST_AsText()` matches byte-for-byte and `ST_AsGeoJSON()` would require displacing a working, golden-pinned
  parser.**
- **H3 parent hierarchy is already materialized** — `hex_cell.res6_parent`/`res5_parent NOT NULL`
  (`postgres_schema.py:418-419`), populated by `h3.cell_to_parent` in
  `domain/economics/substrate/spatial.py:196-197`. **h3-pg's headline coarse/fine clamping value at res 5/6 is
  already served by denormalized columns.**
- **There is no hex tier in the map** — `session.py:1099-1101` raises outside `(county, state, ea)` and `:1107-1108`
  returns `None` for `ea` unconditionally ("`ea` has NO producer — honest absence"). **The MULTI-RESOLUTION
  NATIVE ruling has no renderer path at all today** — that gap, not SQL-side H3 algebra, is the binding
  constraint.
- **h3-pg packaging is nearly free** — `nix eval --raw nixpkgs#postgresql17Packages.h3-pg.version` → **4.2.3**,
  and `flake.nix:150-152` already ships Postgres via `withPackages`, so `ps.h3-pg` is a one-line change, not a
  per-platform C build. Absent from the live instance
  (`pg_available_extensions WHERE name ILIKE '%h3%'` → 0).
- **The Rust client is an in-process PyO3 extension** — `rust/crates/babylon-tui-python/Cargo.toml`:
  `name = "_core"`, `crate-type = ["cdylib"]`, `pyo3 0.29` with `extension-module`; entry point
  `#[pyfunction] fn run(py, host: Py<PyAny>, config_json: &str)`. **Tick-commit push is a direct function
  call** — a Postgres round-trip to notify a component sharing your address space is strictly worse.

### Caveats declared by this lane

- **Version divergence, unflagged until this lane:** the flake ships `postgresql_17` but the live dev DB is
  16.14. **Every measurement here is from 16.14**; plans can change materially across a major, and h3-pg
  availability was verified for `postgresql17Packages`, not 16.
- Recommendation to dedupe reference tables touches PKs on tables the whole economics estate reads, and **LODES
  feeds the Vol II circulation conservation checks** — ADR120 already records a ~0.14%
  `CirculationConservationViolation` traced to LODES/`hex_spatial_map` coverage mismatch. Sequence behind a
  conservation-audit read and **expect a baseline ceremony**.
- Read `0016`, `0018`, `0023`, `0027`, `0029`, `0030` at the cited lines but **not all 32 migrations**;
  `0031`-`0041` were not inspected.
- All measurements from port 5433 only; the 5432 web DB was not inspected.
- `pg_stat_statements` totals are cumulative since an unknown reset point and mix session sizes; the
  ~3 h/campaign extrapolation is **directionally sound but should be re-measured on a single controlled
  nationwide session**.
- Adding `valid_range` changes the on-disk shape of the tables the golden vault renders from — a
  declared-ceremony change with **drift across two separate gate estates**.
- h3-pg introduces a **second H3 implementation** (C 4.2.3) alongside the Python binding (locked 4.5.0); FP
  geodesic ops can disagree at the last bits. Safe only if SQL-side use is restricted to integer hierarchy ops,
  **and that restriction needs a sentinel, not a comment.**

### This lane's Director questions

Is `view_runtime_trace_emission`'s sparse read a perf fix or a defect? Was per-session scoping of reference data
ever intentional (a per-campaign vintage or counterfactual-data mechanic), or incidental? **Is there a target
install footprint for v1.0** (~1.1 GB post-dedupe vs ~5.8 GB is the difference between a plausible and an
implausible download, and it should be a stated budget Phase 4 designs against)? Should Phase 4 charter a **hex
tier for the map pane** (h3-pg only earns adoption if yes)? **AE escalation, flagged not recommended:** should
Postgres-side change capture (wal2json/logical decoding) driving engine events be explored, or ruled out now so
Phase 4 stops considering it? Which PG major is canonical for v1.0? **Is retention actually about disk, or
something else** (save-slot semantics, replay integrity, privacy) — it reclaims only 388 MB, and the design
differs? For the Rust↔Python-observer boundary, push (stateless `pg_notify` carrying only session_id + tick) or
polling `tick_commit` (leaning polling — smaller surface, observer is not latency-critical)?

### Citations

`postgres_schema.py:12,167-169,418-421,525,594,611-615,642,1084-1123,1144`; `pgvector_store.py:186-192`;
`partitioning.py`; `reference/schema.py:126,182,1732`; `domain/geography/h3_mesh.py`;
`domain/economics/substrate/spatial.py:196-197`; `domain/economics/lodes_commute_matrix.py:385`;
`migrations/0016:9,16,26`, `0018:29`, `0023:59-75`, `0027`, `0029:16-24`, `0030:32-37,343,352`, `0036:36`;
`projection/registry.py:41-141`; `rust/crates/babylon-tui/src/views/map.rs:112-113,217-256,832-833,865-867`;
`rust/crates/babylon-tui-python/{Cargo.toml,src/lib.rs:184-185}`; `session.py:1099-1108,1129`;
`uv.lock:3650-3651`; `flake.nix:150-152`; live `db:sql` (`pg_extension`, `pg_database_size`, `pg_class`,
`pg_index`/`pg_am`, `pg_stat_user_indexes`, `pg_stat_statements`, `EXPLAIN`, `pg_available_extensions`,
`pg_views`, `pg_partitioned_table`); `github.com/postgis/h3-pg` + api.md; PG docs `sql-notify`; pgvector docs.

---

## Lane 6 — Graph at rest

**Topic:** graph-in-Postgres for at-rest, queryable, time-travel topology — Apache AGE vs pgrouting vs
relational-native, given the Rust in-memory graph stays sole adjudicator.

### Findings

- **The topology ALREADY round-trips through Postgres relationally, in production shape.**
  `postgres_schema.py:228-315` defines `node_state` (PK `session_id,tick,node_id`, typed columns
  wealth/consciousness/organization_level/… + JSONB `attributes`) and `edge_state` (PK
  `session_id,tick,source_id,target_id,edge_type`, typed value_flow/tension/solidarity_strength/weight + JSONB)
  — **exactly the "relational nodes/edges table" approach the research question asks about**, live at
  `edge_state` 2,686 rows/1.36 MB and `node_state` 2,488 rows/4.70 MB. Indexes already cover the analyst query
  shapes (`idx_node_state_session_tick`, `idx_node_state_session_node`,
  `idx_edge_state_session_tick_source`, …, `:541-566`).
- **A second, older path also exists:** `RuntimeDb.hydrate_graph()` (`runtime_db.py:302-350`) reads
  `node_history`/`edge_history` and **rehydrates a `BabylonGraph` object directly** — proving the graph does
  round-trip through a relational store today (written AND read back into the live graph type).
- **Babylon already has a working relational hyperedge/incidence pattern** — `community_state` (PK
  `session_id,tick,community_type`, the hyperedge-as-row) plus `community_membership` (PK
  `session_id,tick,agent_id,community_type`, the incidence/bipartite join), `:279-315`, with indexes. **The
  "custom composite/incidence table" approach is already validated as a design pattern in this codebase** for
  the n-ary-membership case.
- **Amendment D constrains the AT-REST schema too** (`ai/bsl-architecture-standard.md:37`, ADR/AE clause vi):
  hyperedges must be NATIVE first-class objects in babylon-graph's exposed type system, with Levi/incidence
  sanctioned **ONLY as internal storage, never exposed**, and clique expansion explicitly banned (`:450`: "No
  BSL verb converts a member list into pairwise edges"). So an incidence table is fine as persistence-layer
  storage (Postgres never adjudicates), **but the query/projection layer must present hyperedges as
  first-class objects** — not let callers decompose them the way `community_membership` currently permits
  without explicit framing.
- **Apache AGE:** Apache-2.0, openCypher property graphs, PG 11–18 (so 16 is not a blocker); 4,710 stars, 189
  open issues, last push 2026-07-17, not archived. *(Editor: this lane's inventory claimed AGE was still an
  Apache **incubator** project; this lane corrected it — graduated **top-level 2022-05-17**, verified V10.)*
- **AGE's model is structurally DYADIC** — openCypher relationships have exactly one start and one end node;
  there is **no native n-ary/hyperedge relationship type**. Representing a hyperedge requires the same
  reification SQL already gives free (a Levi/star expansion, structurally identical to
  `community_state`+`community_membership`). **AGE buys nothing hyperedge-native, and it reintroduces exactly
  the pairwise-decomposition shape Amendment D is wary of** if callers query via generic Cypher traversal
  instead of a typed accessor.
- **AGE write amplification:** each vertex/edge becomes a row in AGE's internal `ag_catalog` tables plus an
  `agtype` (JSONB-like binary) payload — a translation/materialization layer on top of what would otherwise be
  a direct `INSERT INTO node_state`. Over a 5,200-tick/30-80 h run, a direct relational insert is the
  lower-amplification path.
- **AGE's core value-add (openCypher multi-hop pattern matching) is available via `WITH RECURSIVE` today** —
  and a grep of `persistence/*.py` + `topology/*.py` found **ZERO recursive-CTE usage**, meaning
  variable-length path / reachability queries are **UNEXPLOITED in the relational layer, not blocked by an
  extension gap** — a concrete "exploit Postgres more" opportunity with zero new extensions.
- **pgRouting is purpose-built for georeferenced network routing** (Dijkstra/A*/TSP over road-network-shaped
  edge tables with cost columns) — a routing algorithm library over a user-supplied edge table, **not a general
  graph-modeling or hyperedge solution.** Narrowly relevant for hex-adjacency/logistics-corridor shortest-path
  (`hex_cell`, `infrastructure_link_state`, `:411-520`); **orthogonal** to topology-at-rest.
- **Verdict evidence:** given (a) the AE boundary, (b) the existing indexed production relational schema
  already doing this job, (c) AGE's dyadic model buying nothing for hyperedges while adding a translation
  layer, and (d) pgRouting's narrow scope — **extend the relational pattern** (an explicit
  `hyperedge_state`/`hyperedge_membership` pair generalized to Amendment D's hyperedge type rather than
  community-specific) **plus `WITH RECURSIVE` for path/reachability.**
- **No finding argues for crossing the AE boundary; no escalation raised** by this lane.

### Corrections this lane made to its own inventory

1. **AGE is NOT an incubator project** (graduated 2022-05-17). The inventory rejected AGE partly on a **false**
   maturity claim; the rejection must stand on the dyadic/packaging/amplification grounds instead.
2. **The topology is NOT in the atomic per-tick envelope** — `persist_tick_atomic` (`_spec_062.py:270-349`)
   writes 8 families + `tick_commit` in one transaction and **no** node/edge rows; `persist_tick`
   (`_legacy.py:167-172`) opens **its own** transaction. `PerTickTransactionEnvelope` (`envelope.py:74-93`)
   **has no node or edge row fields**, yet is designated seam 10, "PORTED as the kernel replay unit"
   (`bsl-architecture-standard.md:559`). **So the adjudicating topology is the one thing not covered by the
   tick's commit marker**, and `_legacy.py:295` then resolves ticks via
   `SELECT MAX(tick) FROM node_state` — the documented anti-pattern applied to the graph, able to **rehydrate
   from a torn tick.**
3. **`node_state` is full-frame every tick** — 92 nodes × 17 ticks = 1,564 rows; 1.69 kB/row → ~156 kB/tick;
   edges 0.51 kB/row × 99 → ~51 kB/tick. *(Editor: the "≈46% of the 452 kB/tick budget" composition is
   invalid — see C1.)*
4. **JSONB is 88% of node heap, stored inline uncompressed** — 3,706 kB of 4,216 kB, `max(pg_column_size)`
   1,846 B, **nothing crosses the ~2 kB TOAST threshold**.
5. **Row-level delta buys nothing on nodes** (1,472 of 1,472 differ = 100.0%) **because churn is 2 keys of
   ~40** — `dpd_state` (256 B) + `dependency_ratio` on 81/81 nodes every tick, `mass_receptivity` 40/81, while
   **`wealth` changes on 2 of 85** and `population` on 3 of 85; 110 distinct keys, avg 40.1/node → **~80% of
   every node row is rewritten byte-identical**. Edges delta well (723 of 1,584 = 45.6% changed, **54.4% pure
   duplicates**).
6. **`trace_log` is neither UNLOGGED nor extant** — the claim traces to a docstring
   (`postgres_schema.py:12`); live `relpersistence <> 'p'` → **0 rows**.
7. **No index anywhere has `(h3_index, tick)` ordering** — `pg_indexes WHERE indexdef LIKE '%h3_index, tick%'`
   → **0**. The PK is `(session_id, tick, h3_index)`; the plan's `Sort Key: h.h3_index, h.tick` is exactly the
   index that does not exist. **Column order, not a missing index per se.**
8. **`node_state`/`edge_state` are NOT in `PARTITIONED_TABLES`** (`partitioning.py:33-43` lists 9 families) —
   confirmed live `relkind='r'`. **So retention currently requires a mass `DELETE` of the graph tables** —
   precisely the bloat/autovacuum stall the module documents avoiding (`:15-18`).
9. **Amendment D's own rulings dictate the at-rest shape:** S-10 whole-hyperedge replacement (`:600`), D25
   member list **is a set**, order never observable (`:714`), S-22 closed registries (`:612`). **So a
   `(tick, hyperedge, member)` incidence table keyed for independent mutation is the WRONG shape** — it makes
   representable the partial mutation S-10 declares unrepresentable. Content-address the member set instead, so
   an unchanged hyperedge costs **zero** member rows per tick.
10. **`edge_type` is stored lowercase** (`tenancy`/`presence`/`exploitation`/`solidarity`/`wages`) against the
    uppercase vocabulary — normalize to the closed registry while rebuilding.
11. **There is no `babylon-graph` crate yet** — `rust/crates/` holds `babylon-kernel`, `babylon-bsl` (both
    scaffolds), `babylon-md`, `babylon-tui`, `babylon-tui-python`. **The at-rest contract is genuinely
    greenfield and can be specified before the crate exists rather than retrofitted.**
12. **Python stays the glue** — `bsl-architecture-standard.md:480-485` rules Python remains the glue for
    "database connections and persistence glue" (ADR174 makes the periphery list illustrative, not
    exhaustive), so the clean seam is **Rust emits the envelope; Python persists it** — keeping
    `sqlx`/`tokio-postgres` + a TLS stack out of the engine's dependency graph.

### Caveats declared by this lane

- Did not benchmark AGE write/query latency — the amplification claim is **architectural** (from vendor docs
  and typical property-graph-on-SQL patterns), **not measured** (installing AGE would violate the read-only
  constraint).
- Could not confirm whether `hypergraph-rs` (the paused external port) bears on a Rust-side
  persistence-adjacent serialization contract — that repo is outside the working directory.
- **The `node_history`/`edge_history` vs `node_state`/`edge_state` split was observed but not reconciled** —
  unclear whether the former are dead/legacy-only or still exercised. **Needs a caller grep before any Phase-4
  design treats one as authoritative and the other as retire-able.**
- Did not inventory the Path A deletion scope against these findings — this lane answers the
  modeling-technology question but does not resolve which of the 20+ `persistence/` files are in scope.
- Field-scoped delta measurements come from **ONE 17-tick session** with 92 nodes and 99 edges; the
  80%-redundancy figure **should be re-measured on a long run** — churn patterns may differ once
  electoral/doctrine systems are exercised over years.
- Content-addressed digests need a **specified byte layout, not an implied one** (CLAUDE.md: language-agnostic
  to the byte) — ascending order plus a declared separator and encoding, or Rust and Python compute different
  digests for the same set.
- Splitting volatile keys out of JSONB changes node attribute shape, which `check:vocabulary` polices (3 rules)
  — expect sentinel work and `ATTRIBUTE_EXEMPTIONS` updates.
- **Escalation flags raised by this lane:** (i) if as-of views become the source for `hydrate_graph`, SQL
  evaluation semantics (LEAD interval math, float round-trips, row ordering) become load-bearing for the
  engine's starting state, colliding with S-2/S-14/S-15 — **loading a save is legitimate; adjudicating from a
  SQL-computed reconstruction is the boundary**, and hydrating only from checkpoint (full-frame) ticks
  sidesteps interval math entirely; (ii) a **custom C extension is the option that invites adjudication into
  the DB** — if operators/functions the engine relies on for graph semantics ship in the DB, the algebra starts
  living there (amendment territory, and a reason to recommend against it).

### This lane's Director questions

May the Rust engine hydrate **only from full-frame checkpoint ticks** so SQL interval math never enters the
determinism path (resolves the escalation cleanly, costs at most one year of replay)? Does "persistence glue
stays Python" mean the engine ships with **no** Postgres client, emitting envelopes across FFI (packaging-cheapest,
but puts an FFI serialization step on every tick's hot path)? **Is the torn-tick defect in scope for Phase 4
Path A work, or fixed now against the frozen Python engine** with Director sign-off per AE clause (viii)?
Should `community_state`/`community_membership` migrate onto the generalized hyperedge tables or stay a special
case (same pattern, and today they expose raw incidence)? Player durability with `synchronous_commit=off`?
Under MULTI-RESOLUTION, when a region refines mid-campaign, is the newly-refined hex set a **new full-frame
checkpoint** or deltas against an absent baseline (determines whether the 52-tick cadence needs a
refinement-triggered extra checkpoint)? When a session is exported and its partitions dropped, do the
content-addressed hyperedge member sets get exported too, or is the parquet export a denormalized flat form
that **discards the digest sharing**?

### Citations

`postgres_schema.py:228-315,411-520,541-585`; `runtime_db.py:302-350`;
`postgres_runtime/_spec_062.py:270-349`; `postgres_runtime/_legacy.py:167-172,295`; `envelope.py:74-93`;
`partitioning.py:14-19,33-43,115-137`; `ai/bsl-architecture-standard.md:37,450,480-485,559,598-600,612,714`;
`migrations/0026,0029:16-24,0030,0036:34-38`; `rust/Cargo.toml`;
`specs/064-…/trace_csv_schema.yaml`; live `db:sql` (`SHOW server_version`, `pg_extension`,
`pg_stat_user_tables`, `pg_class.relkind`, `pg_indexes`, `node_state`/`edge_state` counts + `pg_column_size`,
`lag()` churn analysis, `jsonb_each` key-level churn); `github.com/apache/age`;
`incubator.apache.org/projects/age.html`; `age.apache.org/age-manual/master/{intro/setup,clauses/create}.html`;
`tigerdata.com/learn/postgresql-extensions-pgrouting`.

---

*End of brief.*
