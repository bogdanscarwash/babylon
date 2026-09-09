# H3-in-Postgres delegation — implementation brief

> STATUS: BD-RULED 2026-07-21 ("fantastic idea and we must implement this"). Charters the
> PG-side H3 ruling into concrete units. Analysis verified at HEAD by the 2026-07-21 fork
> sweep (read-only). Fits [PG-maximalism direction] + wiring doctrine W-G + Material
> Triad W1/W5. Execution post-cascade; the schema unit is CEREMONY-GATED.

## What is already true (verified — don't rebuild it)

- PG already owns hex state entirely: `hex_state` sparse delta-persisted keyed
  `(session_id, tick, h3_index)`, `v_hex_state_asof` as-of view, 52-tick checkpoint
  compaction (spec-089). The as-of view IS the cache layer.
- The engine graph stamps ZERO hex nodes (#39/Amendment U, ScaleAdjunction) — in-engine
  hex memory strain is nil. Three dead hex-query call sites remain (Material Triad W1
  item 5 disposition; incl. `simulation_engine.py:235`).
- **No tick-time H3 math exists anywhere** (grepped every engine system for
  `grid_disk`/`k_ring`/`cell_to_parent`/`latlng_to_cell`: nothing). H3 topology runs
  once in build/seed pipelines and is persisted — invariance already exploited.
- Physics correction recorded: H3 indexes are bit-packed 64-bit IDENTIFIERS (resolution
  + base cell + child digits), not numbers you multiply. The wins are integer equality
  joins, B-tree width, parent-by-bit-truncation, precomputed adjacency tables.

## The units

**U1 — BIGINT H3 key migration (THE new lever; ceremony-gated).** The schema stores H3
keys as `VARCHAR(15/16/17)` (`postgres_schema.py:412,442,458,475,635,884…`). Migrate
`hex_cell`, `hex_state`, the r8 reference tables and composite PKs to native `BIGINT`:
~halves key/index width, int-compare joins, cheaper composite PKs. Scope: schema +
persistence round-trip + reference artifacts. This REBUILDS the reference DB —
sha-pinned dataBuild ceremony + re-bless, declared, never drift. Includes the
loader/artifact writers and the graph-bridge readers; conversion at the boundary
(`h3.str_to_int`/`int_to_str`) so Python-side call sites keep their string API where
convenient. SLOT: with T7-beta's embedded-cluster work OR Material Triad W5's county-B
construction — whichever fires first; NOT before the cascade.

**U2 — SQL-side aggregation discipline (mostly already chartered — dedupe, don't
duplicate).** Hex→county rollups are W-G motions on indexed bridge tables; the engine
sees county grain only. The choropleth re-sum refactor (runbook post-merge unit) is the
type case and stays where it is. New work here is only: a declared materialized-rollup
lane IF profiling shows a hot read (profile first, per the bottleneck ruling).

**U3 — h3-pg extension (optional; decision at T7-beta).** We ship the cluster, so the
extension is distributable by construction (PostGIS precedent,
`postgres_schema.py:167-169`). With zero runtime H3 math today it earns its place only
when a live SQL-side H3 op exists (likely W5's county-B zonal work or the β-simplex map
lens). Evaluate nixpkgs availability at T7-beta; do not pre-install unused extensions.

## The hard boundary (Tier A — non-negotiable)

SQL float aggregation has NO pinned evaluation order (scan order / parallel plans).
Anything feeding the tick hash keeps deterministic ORDER BY discipline, uses `numeric`,
or stays in-engine. PG owns invariant substrate lookups and the projection lane; the
engine keeps the physics. Every pushed-down aggregate declares its determinism posture
in the wiring registry row (W-G/W-P motion + sentinel, per ADR109).

## Sequencing

Post-cascade only. U1 rides T7-beta or Material Triad W5 (first to fire); U2 is already
queued via the choropleth unit; U3 decides at T7-beta. Nothing here blocks Gate 3.
