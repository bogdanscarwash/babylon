# Postgres Specification: Babylon Runtime Database

**Status**: Historical draft v3 — superseded by the Rust persistence cutover
**Date**: 2026-03-01
**Depends On**: Constitution, ADR030/031/032/033, Features 011–032
**Supersedes**: SQLite runtime database (`babylon/persistence/runtime_db.py`, `runtime_schema.py`)

> This prompt records the retired Python/Django proposal. It is not current
> architecture. The live boundary is defined by
> `contracts/rust_persistence_cutover_v2.yaml`,
> `contracts/h3_estate_contract_v1.yaml`, and `docs/reference/persistence.rst`.

---

## 1. Scope

This spec defines the PostgreSQL database that replaces all non-reference runtime storage. It covers five concerns:

1. **Game management** — user accounts, sessions, turn submission (Django ORM)
2. **Simulation state** — tick-keyed persistence of the full WorldState graph, including all node types, edges, hypergraph community layer, tick dynamics, contradiction fields, and OODA action results
3. **Trace logging** — detailed execution traces for debugging and coefficient fine-tuning (UNLOGGED partitioned table)
4. **Archival pipeline** — completed games exported to Parquet, uploaded to R2, purged from Postgres
5. **Extensions** — PostGIS (spatial), pgvector (semantic search, replacing ChromaDB)

SQLite stays as the **read-only reference database** (`marxist-data-3NF.sqlite`). It holds immutable federal data: QCEW, BEA, Census, FRED, HIFLD, BTS, FCC, ATUS. The engine hydrates from SQLite at initialization, runs in-memory via NetworkX during ticks, and persists to Postgres between ticks. No SQLite writes occur at runtime.

### 1.1. What This Spec Covers

Everything that currently lives in `RuntimeDatabase` (ADR030) plus everything that currently persists only in-memory via `WorldState.to_graph()`/`from_graph()` serialization:

- Graph node state: SocialClass, Territory, Organization (4 subtypes via discriminated union), KeyFigure
- Graph edge state: all EdgeType values including the 5 org-topology additions (MEMBERSHIP, RECRUITMENT, EMPLOYMENT, COMMAND, PRESENCE)
- Graph metadata: GlobalEconomy, StateFinance, tick_dynamics (NationalTickParameters, SmoothedCoefficients)
- Hypergraph state: CommunityState (14 types), CommunityMembership, CommunityConsciousness
- Hex spatial state: HexEconomicState per H3 cell
- Contradiction fields: field values, spatial/temporal derivatives, Ollivier-Ricci curvature
- OODA resolution: action submissions, initiative scores, action results, Layer 3 deltas
- Events: append-only simulation event ledger
- Replay: tick log with RNG state, mutation summaries, per-system timings
- Aggregates: pre-computed tick summaries for time-series endpoints
- RAG: document chunks with vector embeddings (replacing ChromaDB)
- Execution traces: per-system, per-formula trace logs for debugging and fine-tuning

### 1.2. What This Spec Does NOT Cover

- The SQLite reference database schema (unchanged, read-only)
- In-memory computation during ticks (NetworkX, XGI — unchanged)
- WorldState/Pydantic model definitions (unchanged)
- SimulationEngine system ordering (unchanged)
- Django URL routing, serializers, or view logic (separate spec)
- React frontend (separate spec)

---

## 2. Infrastructure Architecture

### 2.1. Single Database, Single Process

One Postgres server process on the VPS. One database named `babylon`. All tables live in this database.

Rationale: Django needs to read simulation state for API endpoints (`node_state` for the game view, `hex_state` for the map, `tick_summary` for sparklines). The simulation engine needs to read `game_turn` for player actions. If these were in separate databases, every cross-boundary query would require `dblink` or `postgres_fdw` — slow, awkward, and fragile. A single database means `game_turn.session_id` can FK directly to `game_session.id`, and Django's `django.db.connection` can issue raw SQL against any table.

Separate schemas (namespaces within the database) are an option for logical separation if the table count grows unwieldy, but at beta scale everything lives in `public`. The Django ORM tables and raw simulation tables coexist without conflict because Django's migration system only touches the models it manages.

### 2.2. Extensions

```sql
CREATE EXTENSION IF NOT EXISTS postgis;    -- spatial queries on hex geometries
CREATE EXTENSION IF NOT EXISTS vector;     -- pgvector for RAG embeddings
-- Optional, deferred to post-beta:
-- CREATE EXTENSION IF NOT EXISTS age;     -- Apache AGE for Cypher graph queries
```

### 2.3. Connection Architecture

Two connection paths to the same database:

**Django ORM** (`django.db.backends.postgresql`): Manages `auth_user`, `game_session`, `game_turn`. Reads simulation tables via `django.db.connection` for API endpoints. Standard connection pooling via Django's `CONN_MAX_AGE` or `django-db-connection-pool`.

**psycopg direct** (`psycopg` pool): Used by `PostgresRuntime` for bulk simulation writes — 1,500+ hex_state rows per tick, hundreds of node/edge rows. Raw SQL with `executemany()` and `COPY` for performance. This bypasses Django's ORM overhead entirely. Same database, same credentials, separate connection pool.

```python
# Django settings.py
DATABASES = {
    'default': {
        'ENGINE': 'django.db.backends.postgresql',
        'NAME': 'babylon',
        'HOST': 'localhost',
        'PORT': '5432',
    }
}

# PostgresRuntime uses its own pool
import psycopg_pool
pool = psycopg_pool.ConnectionPool("dbname=babylon host=localhost", min_size=2, max_size=5)
```

---

## 3. Design Principles

### 3.1. Tick-Keyed Temporal Tables

The `(session_id, tick, entity_id)` composite key pattern from ADR031 maps directly to Postgres. The fundamental query — "give me the state at tick N for game X" — works the same way. All simulation tables use this pattern. `session_id` scopes every row to a game instance.

### 3.2. JSONB for Flexibility, Columns for Speed

Node and edge attributes go into JSONB columns (same as the SQLite `attributes TEXT` column, but natively queryable via GIN indexes and `->>`/`@>` operators). Frequently-queried scalars get promoted to indexed columns alongside the JSONB blob. The engine writes both — promoted columns are redundant but fast. The full `attributes` JSONB remains source of truth.

A scalar gets its own column if: it appears in a WHERE clause of an analytical query, it appears in an aggregate, it's needed for a time-series endpoint, or it's a discriminator for node/edge subtypes.

### 3.3. Django ORM for Management, Raw SQL for Simulation

Django manages User, GameSession, GameTurn through its ORM. Tick-keyed simulation tables use raw SQL via `psycopg` — the same pattern as the current `sqlite3` usage in `RuntimeDatabase`. This is a deliberate split: the ORM's overhead is unacceptable for bulk writes of 1,500+ hex rows per tick.

### 3.4. No DB I/O During Tick

The constitution's rule (II.6) still holds. Postgres is the persistence layer, not the computation layer. The engine hydrates the full state into memory at tick start, runs all systems in-memory, then persists the result.

### 3.5. Full Snapshots, Not Diffs

Each tick persists a complete snapshot of all state. Same strategy as the SQLite runtime (ADR031). Diffs would save ~80% storage but complicate state reconstruction and replay. Storage is cheap. Correctness is not.

---

## 4. Schema: Game Management Layer (Django ORM)

These tables are defined as Django models. Managed by `django.db.migrations`.

### 4.1. `auth_user` (Django built-in)

Django's default user model. No extension needed for beta.

### 4.2. `game_session`

| Column | Type | Constraints | Notes |
|---|---|---|---|
| `id` | `UUID` | PK, default `gen_random_uuid()` | |
| `player_id` | `INTEGER` | FK → `auth_user.id`, NOT NULL | |
| `scenario` | `VARCHAR(64)` | NOT NULL | Scenario factory name (e.g., `detroit_tri_county`) |
| `current_tick` | `INTEGER` | NOT NULL, default 0 | |
| `status` | `VARCHAR(16)` | NOT NULL, default `'active'` | `active`, `paused`, `completed`, `abandoned`, `archived` |
| `config_json` | `JSONB` | NOT NULL | Serialized `SimulationConfig` |
| `game_defines_json` | `JSONB` | NOT NULL | Serialized `GameDefines` (full model_dump) |
| `trace_level` | `VARCHAR(8)` | NOT NULL, default `'NONE'` | `NONE`, `SUMMARY`, `DEBUG`, `TRACE` |
| `rng_seed` | `BIGINT` | NOT NULL | For deterministic replay |
| `created_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | |
| `updated_at` | `TIMESTAMPTZ` | NOT NULL, auto-update | |

Index: `ix_game_session_player` on `(player_id, status)`.

The `config_json` and `game_defines_json` columns store the full configuration so any game can be reconstructed from its session record. This replaces the SQLite `simulation_metadata` key-value table. `trace_level` controls execution trace verbosity per session.

### 4.3. `game_turn`

| Column | Type | Constraints | Notes |
|---|---|---|---|
| `id` | `BIGSERIAL` | PK | |
| `session_id` | `UUID` | FK → `game_session.id`, NOT NULL | |
| `tick` | `INTEGER` | NOT NULL | |
| `org_id` | `VARCHAR(64)` | NOT NULL | Acting organization |
| `verb` | `VARCHAR(16)` | NOT NULL | One of the 9 player verbs (Constitution V) |
| `action_type` | `VARCHAR(32)` | | Resolved ActionType enum (Feature 032 decomposition) |
| `target_id` | `VARCHAR(64)` | | Target node (nullable) |
| `target_community` | `VARCHAR(32)` | | Target CommunityType if action targets a community |
| `params_json` | `JSONB` | | Verb-specific parameters |
| `submitted_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | |
| `resolved` | `BOOLEAN` | NOT NULL, default `FALSE` | Set TRUE after tick execution |

Index: `ix_game_turn_session_tick` on `(session_id, tick)`.
Unique constraint: `(session_id, tick, org_id)` — one action per org per tick.

---

## 5. Schema: Simulation State Layer

These tables are **not** managed by Django's ORM. Created via `RunSQL` in a Django migration and accessed via `psycopg` directly.

### 5.1. `node_state`

Replaces SQLite `node_history`. Full snapshot of each graph node per tick per game. Covers all four `_node_type` values from `WorldState.to_graph()`: `social_class`, `territory`, `organization`, `key_figure`.

| Column | Type | Constraints | Notes |
|---|---|---|---|
| `session_id` | `UUID` | FK → `game_session.id`, NOT NULL | |
| `tick` | `INTEGER` | NOT NULL | |
| `node_id` | `VARCHAR(64)` | NOT NULL | |
| `node_type` | `VARCHAR(16)` | NOT NULL | |
| `attributes` | `JSONB` | NOT NULL | Full `model_dump()` |
| `wealth` | `NUMERIC` | | Promoted: SocialClass |
| `consciousness` | `FLOAT` | | Promoted: SocialClass |
| `organization_level` | `FLOAT` | | Promoted: SocialClass |
| `class_position` | `VARCHAR(32)` | | Promoted: SocialClass (ClassPosition enum) |
| `population` | `INTEGER` | | Promoted: Territory |
| `profit_rate` | `FLOAT` | | Promoted: Territory |
| `sector_type` | `VARCHAR(32)` | | Promoted: Territory (SectorType enum) |
| `org_type` | `VARCHAR(32)` | | Promoted: Organization (OrgType discriminator) |
| `class_character` | `VARCHAR(32)` | | Promoted: Organization |
| `cohesion` | `FLOAT` | | Promoted: Organization |
| `legal_standing` | `VARCHAR(16)` | | Promoted: Organization |
| `is_institution` | `BOOLEAN` | | Promoted: Organization |

Primary key: `(session_id, tick, node_id)`.

Indexes:
- `ix_node_state_tick` on `(session_id, tick)` — "all nodes at tick N"
- `ix_node_state_entity` on `(session_id, node_id)` — "this entity's history"
- `ix_node_state_type` on `(session_id, tick, node_type)` — "all organizations at tick N"
- `ix_node_state_org_type` on `(session_id, tick, org_type)` WHERE `node_type = 'organization'` (partial index)
- GIN index on `attributes`

Most promoted scalars are NULL for irrelevant node types. `profit_rate` is NULL for organizations; `org_type` is NULL for territories. Promoted columns are query accelerators, not schema enforcement.

### 5.2. `edge_state`

Replaces SQLite `edge_history`. Covers all EdgeType values.

| Column | Type | Constraints | Notes |
|---|---|---|---|
| `session_id` | `UUID` | FK → `game_session.id`, NOT NULL | |
| `tick` | `INTEGER` | NOT NULL | |
| `source_id` | `VARCHAR(64)` | NOT NULL | |
| `target_id` | `VARCHAR(64)` | NOT NULL | |
| `edge_type` | `VARCHAR(32)` | NOT NULL | EdgeType enum value |
| `edge_mode` | `VARCHAR(16)` | | EdgeMode enum (5 modes) |
| `attributes` | `JSONB` | NOT NULL | |
| `value_flow` | `NUMERIC` | | Promoted: economic flow edges |
| `tension` | `FLOAT` | | Promoted: contradiction edges |
| `solidarity_strength` | `FLOAT` | | Promoted: SOLIDARITY edges |
| `weight` | `FLOAT` | | Promoted: MEMBERSHIP edges (population count) |

Primary key: `(session_id, tick, source_id, target_id, edge_type)`.

Indexes:
- `ix_edge_state_tick` on `(session_id, tick)`
- `ix_edge_state_mode` on `(session_id, tick, edge_mode)`
- `ix_edge_state_type` on `(session_id, tick, edge_type)`
- `ix_edge_state_source` on `(session_id, tick, source_id)`

`edge_type` is VARCHAR, not a Postgres ENUM, so new edge types from future features don't require schema migration.

### 5.3. `graph_metadata`

Persists the `G.graph` dict from `WorldState.to_graph()` — GlobalEconomy, StateFinance, tick_dynamics national parameters. Per-session-per-tick state that doesn't belong on nodes or edges.

| Column | Type | Constraints | Notes |
|---|---|---|---|
| `session_id` | `UUID` | FK → `game_session.id`, NOT NULL | |
| `tick` | `INTEGER` | NOT NULL | |
| `economy` | `JSONB` | NOT NULL | Serialized `GlobalEconomy.model_dump()` |
| `state_finances` | `JSONB` | NOT NULL | `{state_id: StateFinance.model_dump()}` |
| `tick_dynamics` | `JSONB` | | NationalTickParameters + SmoothedCoefficients |
| `extra` | `JSONB` | | Catch-all for future graph-level metadata |

Primary key: `(session_id, tick)`.

### 5.4. `community_state`

Persists the XGI hypergraph layer. Currently in-memory only. Each community hyperedge gets a row per tick.

| Column | Type | Constraints | Notes |
|---|---|---|---|
| `session_id` | `UUID` | FK → `game_session.id`, NOT NULL | |
| `tick` | `INTEGER` | NOT NULL | |
| `community_type` | `VARCHAR(32)` | NOT NULL | CommunityType enum |
| `category` | `VARCHAR(32)` | NOT NULL | HyperedgeCategory |
| `heat` | `FLOAT` | NOT NULL | |
| `cohesion` | `FLOAT` | NOT NULL | |
| `infrastructure` | `FLOAT` | NOT NULL | |
| `visibility` | `FLOAT` | NOT NULL | |
| `legal_status` | `VARCHAR(32)` | NOT NULL | LegalStatus enum |
| `reproduction_cost_modifier` | `FLOAT` | NOT NULL | |
| `rent_access_modifier` | `FLOAT` | NOT NULL | |
| `collective_identity` | `FLOAT` | NOT NULL | Feature 029 |
| `dominant_tendency` | `VARCHAR(32)` | NOT NULL | ConsciousnessTendency enum |
| `ideological_contestation` | `FLOAT` | NOT NULL | Feature 029 |
| `infiltration_resistance` | `FLOAT` | | Feature 029 (derived) |

Primary key: `(session_id, tick, community_type)`.

14 rows per tick. Trivial volume.

### 5.5. `community_membership`

Hyperedge incidence records — which agents belong to which communities.

| Column | Type | Constraints | Notes |
|---|---|---|---|
| `session_id` | `UUID` | FK → `game_session.id`, NOT NULL | |
| `tick` | `INTEGER` | NOT NULL | |
| `agent_id` | `VARCHAR(64)` | NOT NULL | |
| `community_type` | `VARCHAR(32)` | NOT NULL | |
| `role` | `VARCHAR(32)` | NOT NULL | MembershipRole enum |
| `strength` | `FLOAT` | NOT NULL | |
| `visibility` | `FLOAT` | NOT NULL | |
| `overt` | `BOOLEAN` | NOT NULL | |

Primary key: `(session_id, tick, agent_id, community_type)`.

Indexes:
- `ix_membership_community` on `(session_id, tick, community_type)`
- `ix_membership_agent` on `(session_id, tick, agent_id)`

### 5.6. `action_result`

OODA action resolution outcomes (Feature 032).

| Column | Type | Constraints | Notes |
|---|---|---|---|
| `id` | `BIGSERIAL` | PK | |
| `session_id` | `UUID` | FK → `game_session.id`, NOT NULL | |
| `tick` | `INTEGER` | NOT NULL | |
| `org_id` | `VARCHAR(64)` | NOT NULL | |
| `action_type` | `VARCHAR(32)` | NOT NULL | ActionType enum |
| `target_id` | `VARCHAR(64)` | | |
| `target_community` | `VARCHAR(32)` | | |
| `initiative_score` | `FLOAT` | NOT NULL | |
| `action_cost` | `FLOAT` | NOT NULL | Action points consumed |
| `success` | `BOOLEAN` | NOT NULL | |
| `consciousness_delta` | `FLOAT` | | CI change applied |
| `heat_delta` | `FLOAT` | | Heat change applied |
| `details` | `JSONB` | | Full resolution details |

Index: `ix_action_result_tick` on `(session_id, tick)`.
Index: `ix_action_result_org` on `(session_id, org_id)`.

### 5.7. `contradiction_field`

Dialectical field topology (Feature 002). Forward-looking — Feature 002 is specced but not implemented. Schema may adjust during implementation.

| Column | Type | Constraints | Notes |
|---|---|---|---|
| `session_id` | `UUID` | FK → `game_session.id`, NOT NULL | |
| `tick` | `INTEGER` | NOT NULL | |
| `node_id` | `VARCHAR(64)` | NOT NULL | |
| `field_name` | `VARCHAR(32)` | NOT NULL | `exploitation`, `immiseration`, `imperial_rent`, `displacement` |
| `value` | `FLOAT` | NOT NULL | |
| `laplacian` | `FLOAT` | | Graph Laplacian |
| `dt` | `FLOAT` | | Temporal first derivative |
| `d2t` | `FLOAT` | | Temporal second derivative |

Primary key: `(session_id, tick, node_id, field_name)`.

### 5.8. `edge_curvature`

Ollivier-Ricci curvature per edge (Feature 002). Recomputed only on topology change.

| Column | Type | Constraints | Notes |
|---|---|---|---|
| `session_id` | `UUID` | FK → `game_session.id`, NOT NULL | |
| `tick` | `INTEGER` | NOT NULL | |
| `source_id` | `VARCHAR(64)` | NOT NULL | |
| `target_id` | `VARCHAR(64)` | NOT NULL | |
| `curvature` | `FLOAT` | NOT NULL | |
| `gradient` | `JSONB` | | Per-field gradients along this edge |

Primary key: `(session_id, tick, source_id, target_id)`.

### 5.9. `simulation_event`

Replaces SQLite `events`. Append-only ledger covering all EventType values.

| Column | Type | Constraints | Notes |
|---|---|---|---|
| `id` | `BIGSERIAL` | PK | |
| `session_id` | `UUID` | FK → `game_session.id`, NOT NULL | |
| `tick` | `INTEGER` | NOT NULL | |
| `event_type` | `VARCHAR(48)` | NOT NULL | |
| `entity_id` | `VARCHAR(64)` | | |
| `community_type` | `VARCHAR(32)` | | |
| `details` | `JSONB` | NOT NULL | |
| `created_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | |

Indexes:
- `ix_event_session_tick` on `(session_id, tick)`
- `ix_event_type` on `(session_id, event_type)`
- `ix_event_community` on `(session_id, community_type)` WHERE `community_type IS NOT NULL` (partial)

### 5.10. `tick_summary`

Pre-aggregated metrics per tick. What the `/api/games/{id}/timeseries/` endpoint reads.

| Column | Type | Constraints | Notes |
|---|---|---|---|
| `session_id` | `UUID` | FK → `game_session.id`, NOT NULL | |
| `tick` | `INTEGER` | NOT NULL | |
| `year` | `INTEGER` | | Simulation year |
| `total_c` | `NUMERIC` | | |
| `total_v` | `NUMERIC` | | |
| `total_s` | `NUMERIC` | | |
| `exploitation_rate` | `FLOAT` | | s/v |
| `profit_rate` | `FLOAT` | | s/(c+v) |
| `imperial_rent` | `FLOAT` | | Φ aggregate |
| `avg_consciousness` | `FLOAT` | | |
| `solidarity_edge_count` | `INTEGER` | | |
| `antagonistic_edge_count` | `INTEGER` | | |
| `co_optive_edge_count` | `INTEGER` | | |
| `org_count` | `INTEGER` | | |
| `player_org_count` | `INTEGER` | | |
| `uprising_count` | `INTEGER` | | |
| `repression_count` | `INTEGER` | | |
| `conservation_check` | `BOOLEAN` | | Did value conservation hold? |

Primary key: `(session_id, tick)`.

### 5.11. `tick_log`

Deterministic replay metadata. Replaces SQLite `tick_log`. Now includes per-system timing.

| Column | Type | Constraints | Notes |
|---|---|---|---|
| `session_id` | `UUID` | FK → `game_session.id`, NOT NULL | |
| `tick` | `INTEGER` | NOT NULL | |
| `rng_state` | `BYTEA` | | Serialized RNG state |
| `mutations_json` | `JSONB` | | Mutation summary |
| `invariant_checks` | `JSONB` | | Conservation checks, sum-to-one, etc. |
| `system_timings` | `JSONB` | | `{"ImperialRentSystem": 12, "OODASystem": 45, ...}` (ms) |
| `wall_time_ms` | `INTEGER` | | Total tick execution time |

Primary key: `(session_id, tick)`.

`system_timings` is the lightweight performance data that persists through crashes and across archival. It answers "which system is slow?" without needing the full trace log. The detailed "why is it slow?" investigation goes to the trace table (section 7).

---

## 6. Schema: PostGIS Spatial Layer

### 6.1. `hex_cell`

Static reference table — generated once per scenario from TIGER/Line boundaries. Shared across sessions.

| Column | Type | Constraints | Notes |
|---|---|---|---|
| `h3_index` | `VARCHAR(15)` | PK | H3 res 7 cell ID |
| `county_fips` | `VARCHAR(5)` | NOT NULL | |
| `res6_parent` | `VARCHAR(15)` | NOT NULL | |
| `res5_parent` | `VARCHAR(15)` | NOT NULL | |
| `geometry` | `GEOMETRY(POLYGON, 4326)` | NOT NULL | |
| `centroid` | `GEOMETRY(POINT, 4326)` | NOT NULL | |

Spatial index: GiST on `geometry`.
Index: `ix_hex_cell_county` on `county_fips`.

### 6.2. `hex_state`

Per-tick economic state of each hex.

| Column | Type | Constraints | Notes |
|---|---|---|---|
| `session_id` | `UUID` | FK → `game_session.id`, NOT NULL | |
| `tick` | `INTEGER` | NOT NULL | |
| `h3_index` | `VARCHAR(15)` | FK → `hex_cell.h3_index`, NOT NULL | |
| `constant_capital` | `NUMERIC` | NOT NULL | c |
| `variable_capital` | `NUMERIC` | NOT NULL | v |
| `surplus_value` | `NUMERIC` | NOT NULL | s |
| `employment` | `FLOAT` | NOT NULL | |
| `dept_shares` | `FLOAT[4]` | NOT NULL | Dept I, IIa, IIb, III |
| `profit_rate` | `FLOAT` | NOT NULL | |
| `exploitation_rate` | `FLOAT` | NOT NULL | |

Primary key: `(session_id, tick, h3_index)`.

Indexes:
- `ix_hex_state_tick` on `(session_id, tick)` — the map endpoint
- `ix_hex_state_hex` on `(session_id, h3_index)` — hex time series

~1,500 rows per tick for tri-county Detroit.

---

## 7. Schema: Trace Logging Layer

### 7.1. Purpose

The simulation state tables (sections 5–6) store WHAT happened each tick. The trace log stores WHY — which system produced which mutations, what inputs went into each formula, why the OODA system scored initiative the way it did, how alpha-smoothing affected each coefficient.

The trace layer exists for two use cases: **debugging** (something went wrong, trace back to the formula evaluation that produced the bad value) and **fine-tuning** (adjust GameDefines coefficients by observing how they propagate through the simulation).

### 7.2. Why UNLOGGED

Trace data has fundamentally different durability requirements than game state. If Postgres crashes and trace data is lost, you re-run the game — it's deterministic from the RNG seed in `game_session`. The game state tables are WAL-logged (crash-safe). The trace table is UNLOGGED (no WAL writes), which makes bulk inserts roughly 5-10x faster. UNLOGGED tables are truncated on crash recovery.

### 7.3. `trace_log`

```sql
CREATE UNLOGGED TABLE trace_log (
    id              BIGSERIAL PRIMARY KEY,
    session_id      UUID NOT NULL,
    tick            INTEGER NOT NULL,
    system_name     VARCHAR(48) NOT NULL,
    level           VARCHAR(8) NOT NULL,
    event           VARCHAR(48) NOT NULL,
    node_id         VARCHAR(64),
    data            JSONB NOT NULL,
    ts              TIMESTAMPTZ NOT NULL DEFAULT now()
) PARTITION BY LIST (session_id);
```

Partitioned by `session_id`. When a game session is created with `trace_level != 'NONE'`, `PostgresRuntime` creates a partition:

```sql
CREATE TABLE trace_log_{session_short_id} PARTITION OF trace_log
    FOR VALUES IN ('{session_id}');
```

When debugging is complete or the game is archived, the partition is dropped:

```sql
DROP TABLE trace_log_{session_short_id};
```

Partition drop is instant, creates zero dead tuples, requires no VACUUM.

Indexes (created per-partition automatically):
- `ix_trace_event` on `(session_id, tick, event)`
- GIN index on `data`

### 7.4. Verbosity Levels

Controlled by `game_session.trace_level`. Each level includes everything from lower levels.

**NONE** — No trace rows. Production default. Zero overhead.

**SUMMARY** — One entry per system per tick: system name, wall_time_ms, mutation count, input/output node counts. ~20 rows/tick.

```json
{"system": "ImperialRentSystem", "wall_time_ms": 12, "mutations": 8, "nodes_read": 20, "nodes_written": 8}
```

**DEBUG** — Per-node state changes, formula evaluations with inputs/outputs, edge mode transitions, OODA initiative breakdown, consciousness deltas. ~200-500 rows/tick.

```json
{"formula": "imperial_rent", "node": "SC_WAYNE_PROLETARIAT", "inputs": {"surplus": 142.3, "phi_rate": 0.23}, "output": 32.7}
{"org": "org_detroit_pd", "cycle_time": 3.2, "institutional_bonus": 8.0, "score": 11.7}
{"source": "SC_WAYNE_PROLETARIAT", "target": "SC_OAKLAND_PROLETARIAT", "old_mode": "TRANSACTIONAL", "new_mode": "SOLIDARISTIC", "predicate": "solidarity_threshold_crossed"}
```

**TRACE** — Everything in DEBUG plus intermediate computation values, alpha-smoothing steps, conservation check arithmetic, overlap matrix diffs, coefficient convergence data. ~1,000-2,000 rows/tick.

```json
{"coefficient": "gamma_III", "raw_value": 0.342, "smoothed_value": 0.328, "alpha": 0.15, "delta": -0.014}
{"invariant": "value_conservation", "expected": 1000.0, "actual": 999.97, "residual": 0.03, "within_epsilon": true}
```

### 7.5. TraceRecorder Observer

The engine already has the `SessionRecorder` observer pattern (ADR030). A new `TraceRecorder` observer collects structured trace events during tick execution and flushes them to the trace table after the tick completes. This respects the no-DB-I/O-during-tick rule: events accumulate in an in-memory list during execution, then write once at tick end.

```python
class TraceRecorder:
    """Observer that collects execution traces and flushes to Postgres."""

    def __init__(self, runtime: PostgresRuntime, level: TraceLevel):
        self._runtime = runtime
        self._level = level
        self._buffer: list[dict] = []

    def trace(self, system: str, event: str, data: dict, *, level: TraceLevel = TraceLevel.DEBUG, node_id: str | None = None):
        """Buffer a trace event (called during tick execution)."""
        if level.value > self._level.value:
            return  # Below configured verbosity
        self._buffer.append({"system_name": system, "event": event, "node_id": node_id, "data": data, "level": level.value})

    def flush(self, session_id: UUID, tick: int):
        """Write buffered events to trace_log (called after tick completion)."""
        if not self._buffer:
            return
        self._runtime.persist_traces(session_id, tick, self._buffer)
        self._buffer.clear()
```

Systems access the tracer through the `ServiceContainer` or `TickContext`. When `trace_level` is NONE, the tracer is a no-op stub.

### 7.6. Querying Traces

```sql
-- Why did profit_rate spike at tick 47?
SELECT system_name, event, data
FROM trace_log
WHERE session_id = 'abc-123' AND tick = 47
  AND event = 'formula_eval' AND data->>'formula' = 'profit_rate'
ORDER BY ts;

-- OODA initiative breakdown at tick 47
SELECT data->>'org' AS org,
       (data->>'score')::float AS initiative,
       (data->>'institutional_bonus')::float AS bonus
FROM trace_log
WHERE session_id = 'abc-123' AND tick = 47 AND event = 'initiative_score'
ORDER BY initiative DESC;

-- Alpha-smoothing convergence for gamma_III across all ticks
SELECT tick,
       (data->>'raw_value')::float AS raw,
       (data->>'smoothed_value')::float AS smoothed
FROM trace_log
WHERE session_id = 'abc-123'
  AND event = 'alpha_smooth' AND data->>'coefficient' = 'gamma_III'
ORDER BY tick;

-- Every edge mode transition in the game
SELECT tick, data->>'source' AS src, data->>'target' AS tgt,
       data->>'old_mode' AS old, data->>'new_mode' AS new
FROM trace_log
WHERE session_id = 'abc-123' AND event = 'edge_mode_transition'
ORDER BY tick;
```

---

## 8. Schema: pgvector Semantic Layer

Replaces ChromaDB. The `VectorStore` protocol gets a `PgVectorStore` implementation; `RagPipeline` doesn't change.

### 8.1. `document_chunk`

| Column | Type | Constraints | Notes |
|---|---|---|---|
| `id` | `VARCHAR(128)` | PK | |
| `session_id` | `UUID` | FK → `game_session.id` | Nullable (global theory corpus) |
| `source_file` | `VARCHAR(256)` | NOT NULL | |
| `chunk_index` | `INTEGER` | NOT NULL | |
| `content` | `TEXT` | NOT NULL | |
| `embedding` | `vector(1536)` | NOT NULL | Dimension set at RAG migration time |
| `metadata` | `JSONB` | | |
| `created_at` | `TIMESTAMPTZ` | NOT NULL, default `now()` | |

Index: HNSW on `embedding` with cosine distance.
Index: `ix_chunk_session` on `session_id`.

---

## 9. Hydration and Persistence Flow

### 9.1. Game Initialization

1. Player creates game → Django creates `game_session` row
2. If `trace_level != 'NONE'`, create trace_log partition for this session
3. Engine reads SQLite reference DB to build initial `WorldState`
4. `DefaultSpatialSubstrateSource` generates `HexGrid` from TIGER boundaries
5. `WorldState.to_graph()` → NetworkX graph hydrated with all node types
6. `build_community_hypergraph()` → XGI hypergraph from CommunityMembership records
7. Initial state persisted to Postgres: `node_state`, `edge_state`, `graph_metadata`, `community_state`, `community_membership`, `hex_state` at tick 0
8. `hex_cell` reference table populated (once per scenario, shared across sessions)

### 9.2. Turn Resolution

1. Player submits turn → Django validates → `game_turn` row created
2. Engine loads current tick's state from Postgres:
   - `node_state` → reconstruct nodes by `_node_type` dispatch
   - `edge_state` → reconstruct edges
   - `graph_metadata` → reconstruct economy, state_finances, tick_dynamics
   - `community_state` + `community_membership` → rebuild XGI hypergraph
   - `hex_state` → reconstruct `HexGrid`
   - `game_turn` rows for this tick → inject into `TickContext`
3. `SimulationEngine.run_tick()` executes all systems in-memory (Layer 0 → Action Phase → Layer 3)
4. New state persisted to Postgres (all tables from section 5 + 6)
5. If tracing enabled: `TraceRecorder.flush()` writes to `trace_log`
6. `game_session.current_tick` incremented
7. Django returns new state as JSON to React

### 9.3. Analytical Queries (God Mode)

Hit Postgres directly, never go through the engine:

```sql
-- Profit rate for Wayne County hexes, ticks 0-42
SELECT tick, AVG(profit_rate)
FROM hex_state
WHERE h3_index IN (SELECT h3_index FROM hex_cell WHERE county_fips = '26163')
  AND session_id = $1
GROUP BY tick ORDER BY tick;

-- Community consciousness trajectory
SELECT tick, collective_identity, dominant_tendency
FROM community_state
WHERE session_id = $1 AND community_type = 'new_afrikan'
ORDER BY tick;

-- Bifurcation analysis: solidarity vs consciousness at crisis
SELECT tick, solidarity_edge_count, avg_consciousness, uprising_count
FROM tick_summary
WHERE session_id = $1 AND tick BETWEEN 40 AND 60;
```

---

## 10. Archival Pipeline: Postgres → Parquet → R2 → DuckDB

### 10.1. Motivation

Completed games accumulate in Postgres. A 260-tick game produces ~125MB of state data. Ten concurrent beta testers generate ~1.25GB. This is fine for a VPS, but over months of play the data grows unboundedly. More importantly, the interesting analytical queries — "across all 500 games I've run, what coefficient settings produce bifurcation before tick 100?" — are OLAP workloads that benefit from columnar storage and compression, not Postgres's row-oriented engine.

The archival pipeline moves completed games out of Postgres into Parquet files in R2 blob storage. DuckDB provides the analytical query layer over the archived data.

### 10.2. Why Parquet

Parquet is a columnar file format with typed, self-describing schema and aggressive compression. The same 125MB game in Postgres becomes ~12-15MB in Parquet with zstd compression. Columnar layout means scanning `profit_rate` across a million rows reads only the profit_rate bytes, not every other column. DuckDB, ClickHouse, pandas, and polars all read Parquet natively. The files are portable, versionable, and an LLM can introspect the schema metadata without manual explanation.

### 10.3. Why R2

Cloudflare R2 is S3-compatible with zero egress fees. 1,000 archived games at ~15MB each = 15GB = $0.22/month. The Babylon project already has Cloudflare infrastructure (MCP connector in the project). R2 is essentially free cold storage.

### 10.4. Export Flow

A background job (cron or Django management command) finds sessions where `status = 'completed'` and `updated_at < now() - interval '24 hours'`:

```
1. SELECT * FROM {table} WHERE session_id = $1 ORDER BY tick
   → Stream rows into PyArrow batches (10,000 rows per batch)
   → Write Parquet file per table with zstd compression

2. Upload Parquet files to R2:
   s3://babylon-archives/games/{session_id}/{table_name}.parquet

3. Verify upload (checksum comparison)

4. DELETE FROM {table} WHERE session_id = $1
   (ordered by FK dependencies: trace_log first, game_turn last)

5. UPDATE game_session SET status = 'archived' WHERE id = $1

6. If trace_log partition exists: DROP TABLE trace_log_{session_short_id}
```

The `game_session` row itself is never deleted. It serves as the permanent index of all games ever played, with `status = 'archived'` and the original `config_json`/`game_defines_json` preserved. Archived sessions can be rehydrated from R2 if needed.

### 10.5. Purge and VACUUM

After bulk DELETE, dead tuples accumulate in the affected tables. Autovacuum handles this eventually, but for immediate space reclamation after a batch purge:

```sql
VACUUM ANALYZE node_state, edge_state, hex_state, community_membership;
```

`VACUUM` marks dead tuple space as reusable (fast, no lock). `ANALYZE` updates table statistics for the query planner. `VACUUM FULL` rewrites the table file to return space to the OS (slow, exclusive lock) — only use during scheduled maintenance windows when no games are active.

For higher-volume scenarios (100+ games), use Postgres declarative partitioning on `session_id` for the heavy tables (`node_state`, `edge_state`, `hex_state`). Partition drop is instant with zero dead tuples and no VACUUM needed. Creating a partition per session is a one-line DDL call from `PostgresRuntime`:

```sql
CREATE TABLE node_state_{session_short_id} PARTITION OF node_state
    FOR VALUES IN ('{session_id}');
-- At purge time:
DROP TABLE node_state_{session_short_id};
```

### 10.6. Autovacuum Configuration

Default autovacuum settings are fine for beta. If table sizes grow or write patterns change, tune these per-table:

```sql
ALTER TABLE hex_state SET (
    autovacuum_vacuum_threshold = 5000,       -- trigger after 5000 dead tuples
    autovacuum_vacuum_scale_factor = 0.05,    -- or 5% of table
    autovacuum_analyze_threshold = 1000
);
```

Monitor with:

```sql
SELECT relname, n_live_tup, n_dead_tup, last_vacuum, last_autovacuum
FROM pg_stat_user_tables
WHERE schemaname = 'public'
ORDER BY n_dead_tup DESC;
```

### 10.7. DuckDB for Cross-Game Analytics

DuckDB reads Parquet from R2 natively. No import, no ETL. The Parquet files ARE the analytical database.

```python
import duckdb

con = duckdb.connect()
con.execute("""
    INSTALL httpfs; LOAD httpfs;
    SET s3_endpoint = '<account-id>.r2.cloudflarestorage.com';
    SET s3_access_key_id = '...';
    SET s3_secret_access_key = '...';
""")

# Query across ALL archived games
result = con.execute("""
    SELECT
        ts.session_id,
        gs.game_defines_json->>'economy'->>'extraction_efficiency' AS alpha,
        MIN(e.tick) AS first_uprising_tick,
        AVG(ts.profit_rate) AS mean_profit_rate
    FROM read_parquet('s3://babylon-archives/games/*/tick_summary.parquet') ts
    JOIN read_parquet('s3://babylon-archives/games/*/simulation_event.parquet') e
        ON ts.session_id = e.session_id
    JOIN read_parquet('s3://babylon-archives/games/*/game_session.parquet') gs
        ON ts.session_id = gs.id
    WHERE e.event_type = 'UPRISING'
    GROUP BY ts.session_id, alpha
""").fetchdf()
```

This is the "divine omens from tea leaves" workflow: run games with varied GameDefines → archive to R2 → query with DuckDB → pipe results to an LLM conversation for pattern detection → adjust coefficients → repeat.

DuckDB also reads local Parquet files for development:

```python
# Before uploading to R2, analyze locally
con.execute("""
    SELECT tick, AVG(profit_rate)
    FROM read_parquet('/tmp/archives/abc-123/hex_state.parquet')
    GROUP BY tick ORDER BY tick
""")
```

### 10.8. When to Consider ClickHouse

If the archived corpus grows past what DuckDB handles comfortably (hundreds of millions of rows across thousands of games) or you need real-time trace analytics during live gameplay, ClickHouse is the upgrade path. Single binary, `apt install clickhouse-server`, columnar storage with 10-50x compression, designed for exactly this OLAP workload. ClickHouse ingests Parquet directly and has a Postgres foreign data wrapper. Migration from DuckDB+Parquet to ClickHouse is straightforward. This is a post-launch concern.

---

## 11. Repository Pattern Interface

### 11.1. Protocol

The engine talks to the persistence layer through an abstract interface. The existing `RuntimeDatabase` stays for local dev/testing. `PostgresRuntime` implements the same protocol for production.

```python
class RuntimePersistence(Protocol):
    """Backend-agnostic simulation state persistence."""

    def persist_tick(self, tick: int, graph: nx.DiGraph, events: list[dict] | None = None, *, session_id: UUID | None = None) -> None: ...
    def hydrate_graph(self, tick: int, *, session_id: UUID | None = None) -> nx.DiGraph: ...
    def log_tick(self, tick: int, rng_state: bytes | None = None, mutations: dict | None = None, invariant_checks: dict | None = None, wall_time_ms: int | None = None, system_timings: dict | None = None, *, session_id: UUID | None = None) -> None: ...
    def set_metadata(self, key: str, value: str) -> None: ...
    def get_metadata(self, key: str) -> str | None: ...
```

### 11.2. PostgresRuntime Extensions

Beyond the base protocol, `PostgresRuntime` adds methods for subsystems that didn't exist when `RuntimeDatabase` was written:

- `persist_graph_metadata(tick, economy, state_finances, tick_dynamics, session_id)`
- `persist_community_state(tick, community_states, memberships, session_id)`
- `hydrate_community_state(tick, session_id)` → tuple of states dict and memberships list
- `persist_hex_state(tick, hex_states, session_id)`
- `persist_action_results(tick, results, session_id)`
- `persist_contradiction_fields(tick, fields, curvatures, session_id)`
- `persist_tick_summary(tick, summary, session_id)`
- `persist_traces(session_id, tick, trace_events)` — bulk insert to trace_log
- `create_session_partition(session_id)` — create trace_log partition
- `drop_session_partition(session_id)` — drop trace_log partition
- `export_session_to_parquet(session_id, output_dir)` — archival export

The `Simulation` class receives a persistence handle via `ServiceContainer`. It doesn't know or care which backend.

---

## 12. Migration Strategy

### 12.1. What Changes in the Engine

`RuntimeDatabase` gets a Postgres-backed sibling. `sqlite3` calls become `psycopg` calls. DDL changes: `AUTOINCREMENT` → `SERIAL`, `TEXT` → `VARCHAR`/`TEXT`, add `JSONB`/`vector`/`GEOMETRY` types, add `session_id` scoping to every table.

### 12.2. What Changes in RAG

`VectorStore` gets `PgVectorStore`. `ChromaManager` becomes optional. `RagPipeline` unchanged.

### 12.3. What Changes in Persistence

`persist_tick()` grows to also write `graph_metadata`, `community_state`, `community_membership`, `action_result`, `contradiction_field`, `edge_curvature`. `hydrate_graph()` grows to read them.

### 12.4. What Doesn't Change

- `WorldState` — frozen Pydantic, `to_graph()`/`from_graph()` unchanged
- `SimulationEngine` — takes graph + services + context
- All engine systems — mutate in-memory graph
- `GraphProtocol` — `query_nodes()`, `update_node()` unchanged
- SQLite reference database — read-only data source
- `ServiceContainer` — gains persistence + tracer handles
- XGI community system — in-memory, unaware of persistence

### 12.5. Legacy Tables

SQLite `runtime_schema.py` contains legacy compatibility tables from the DuckDB migration: `agent_state`, `production_event`, `network_edge`, `territorial_control`. These are superseded by the unified tables and do NOT migrate to Postgres. The `simulation_metadata` key-value table is superseded by `game_session.config_json`.

---

## 13. Sizing Estimates

Per game session, per tick, assuming tri-county Detroit (~1,500 hexes, ~20 social class nodes, ~10 orgs, ~50 edges, ~14 communities, ~200 memberships):

| Table | Rows/tick | Row size | Per tick |
|---|---|---|---|
| `node_state` | ~35 | ~2 KB | ~70 KB |
| `edge_state` | ~55 | ~1 KB | ~55 KB |
| `graph_metadata` | 1 | ~5 KB | ~5 KB |
| `community_state` | 14 | ~300 B | ~4 KB |
| `community_membership` | ~200 | ~100 B | ~20 KB |
| `hex_state` | ~1,500 | ~200 B | ~300 KB |
| `action_result` | ~10-30 | ~500 B | ~15 KB |
| `contradiction_field` | ~80 | ~100 B | ~8 KB |
| `edge_curvature` | ~50 | ~100 B | ~5 KB |
| `simulation_event` | ~5-20 | ~500 B | ~10 KB |
| `tick_summary` | 1 | ~200 B | ~200 B |
| `tick_log` | 1 | ~1 KB | ~1 KB |
| `trace_log` (DEBUG) | ~300 | ~300 B | ~90 KB |
| `trace_log` (TRACE) | ~1,500 | ~300 B | ~450 KB |

**Game state per tick** (excluding traces): ~490 KB.
**Over 260 ticks**: ~125 MB per session.
**After Parquet export**: ~12-15 MB per session (zstd compression).

| Scale | Active Postgres | R2 Archive |
|---|---|---|
| 10 beta testers | ~1.25 GB | ~150 MB |
| 100 concurrent games | ~12.5 GB | ~1.5 GB |
| 1,000 completed games archived | 0 (purged) | ~15 GB ($0.22/mo) |

A 20 GB VPS disk handles beta with ample headroom.

---

## 14. Open Questions

1. **Session-level partitioning**: At beta, no partitioning needed for state tables. At 100+ concurrent games, partition `node_state`, `edge_state`, `hex_state` by `session_id`. The trace_log is already partitioned from day one.

2. **Apache AGE**: Gives Cypher queries for complex graph traversals. Recursive CTEs can do the same without AGE. Evaluate after God Mode needs multi-hop graph queries.

3. **Embedding model dimension**: `vector(1536)` matches OpenAI. If switching to local model (e.g., 384 dims), the column changes. Define at RAG migration time.

4. **Contradiction field schema**: Feature 002 is specced but not implemented. `contradiction_field` and `edge_curvature` tables are forward-looking and may adjust during implementation.

5. **ClickHouse timeline**: DuckDB + Parquet is sufficient through beta and likely beyond. Evaluate ClickHouse only if cross-game analytical corpus exceeds ~100M rows or real-time trace analytics become necessary during live gameplay.
