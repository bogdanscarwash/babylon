<!-- Investigation product 2026-07-20 (task #57 prep; PR-committed for the design session).
     Produced by a 5-agent read-only workflow (4 Sonnet sweeps + Opus synthesis) in session
     4d2dbe24; every claim carries a file:line citation from the sweeps. This is the INPUT to
     the brainstorming/design session owner ruling W1 requires before any Wayne code moves. -->

# Design Brief: Wayne County Hex-Substrate Migration (Task #57)

**Status:** input to BD brainstorming session · **Scope guard:** owner ruling W1 (2026-07-19) — #39 county-keyed USScenario only; Wayne stays res-6 sub-county until this design matures · **Source:** four read-only sweeps, all claims cited `file:line` from sweep evidence · **Predates:** 2026-07-20 TUI ruling (browser client = LEGACY; terminal Archive successor; `observe()` projections transport-independent) — scope implications surfaced in §3.

---

## 1. Current-state map

### 1.1 Wayne's res-6 estate (what exists today)

- **81 Territory nodes, one per H3 res-6 cell.** `_legacy_wayne.py:65-89` (`WAYNE_COUNTY_BOUNDS`, `H3_RESOLUTION=6`, comment "~81 hexes"); `_create_wayne_county_territories()` at `_legacy_wayne.py:257-293` builds `h3.polygon_to_cells(polygon, H3_RESOLUTION)` then one `Territory(id=cell, h3_index=cell, ...)` per cell (`:279-291`). Count 81 pinned by `tests/integration/web/test_dashboards.py:170` and `tests/integration/web/test_balkanization_seed.py:65`.
- **Keying is inverted vs USScenario.** Wayne: `id == h3_index ==` raw 15-char H3 string, `county_fips` unset at build time (`_legacy_wayne.py:279-281`). USScenario: opaque `T{i:04d}` id, `h3_index=None`, identity in `county_fips` only (`_legacy.py:697-816`, `h3_index=None` at `:816`). `Territory.id` regex `^(T[0-9]{3,}|[0-9a-f]{15})$` (`models/entities/territory.py:76`) forbids a bare FIPS in id.
- **`county_fips` stamped post-hoc, identically, by the WEB bridge — not the engine scenario.** `web/game/engine_bridge.py:7577` (`WAYNE_COUNTY_FIPS='26163'`), `_seed_wayne_county_fips` at `:7708-7762` stamps all 81 territories with the same FIPS, deliberately collapsing them to one `CountyEconomicState` for county-tick economics while keeping 81 distinct graph nodes for spatial/OODA/map/tenancy.
- **85 edges: 81 TENANCY + 2 EXPLOITATION + 1 WAGES + 1 SOLIDARITY.** `_create_wayne_county_relationships` at `_legacy_wayne.py:386-479`; territories are rent-level-sorted and sliced into **4 non-overlapping zones** (bourgeois top 10% / suburban 10-40% / Dearborn 40-60% / Detroit proletariat bottom 40%, `:443-477`). Total pinned by `tests/integration/web/test_dashboards.py:158-168` and `tests/unit/test_contract_parity.py:252`.
- **Organizations attach via PRESENCE, not TENANCY**, auto-derived by `WorldState.to_graph()` from each org's `territory_ids` (`world_state.py:694-709`); player org takes `detroit_hexes[:2]`, Detroit PD takes `[:3]` → exactly 3 PRESENCE-linked hexes with one shared hub (`_legacy_wayne.py:609-617`, pinned non-degenerate by `tests/unit/web/test_aw4_centrality_lens.py:41-67`). Business NPCs attach the same way via `Business.territory_ids` (`business_seeds.py:53-88`, `_legacy_wayne.py:628`).
- **Registered "declared synthetic"** — hand-authored, never reference-DB-derived (`sentinels/synthetic/registry.py:230-254`), categorically unlike USScenario's real-data artifact (ADR092).

### 1.2 Systems consuming Wayne's topology

Six engine systems query `EdgeType.TENANCY`: `production.py:222` (`_find_tenancy_target`, per-territory extraction intensity), `territory.py:361` (penal-colony org suppression), `contradiction.py:129-197` (per-edge rent-tension), `contradiction_field.py:38-44` (E0 exploitation/atomization field), `epistemic_horizon.py:83-108` (per-territory mass_receptivity), `bifurcation/ceiling.py:239-261` (`_get_tenancy_targets`, territorial-adjacency check). **Dispossession is NOT among them** — `dispossession_events.py:28-101` reads/writes territory `wealth` directly, no edge traversal.

Behavioral note: `bifurcation/ceiling.py:220-236`'s same-territory adjacency check between classes is **always False today** — Wayne's 4 TENANCY zones are non-overlapping slices and Wayne emits **zero ADJACENCY edges** (`sentinels/vocabulary/registry.py:551-553`: ADJACENCY heavily consumed, "never literally produced" by any scenario).

### 1.3 Substrate / scale machinery post-#39

- **`NodeType.HEX` is declared vocabulary only** — no production code stamps a `hex` graph node (`models/enums/topology.py:45-69`; vocabulary sentinel `UNSTAMPED_QUERY_ALLOWLIST = {"community","hex"}`, `registry.py:111`). Constitution Amendment U: "hex is never a graph node" (`CONSTITUTION.md:457`), analogized to community's INV-010 but **not code-enforced** — `rg hex src/babylon/engine/invariants.py` returns zero hits (verified in sweep).
- **`SubstrateSystem` (@2.5)** was rewritten by #39 T6 from a dead `NodeType.HEX` no-op into county-grain `raw_material_stock` depletion + `ScaleAdjunction` lattice binding (`engine/systems/substrate.py:193-260`). It publishes CZ/MSA/state/nation extensive-sum aggregates to `context.persistent_data` (`:141-145`). It **never does a per-tick DB read** (only a one-time lattice build, `:83-88`), and its `raw_material_stock` is a **parallel dollar-denominated family** to `MetabolismSystem`'s `biocapacity`, never joined (`:31-38`).
- **`ScaleAdjunction`** (`domain/dialectics/instances/scale.py:56-177`) is a generic allocate⊣aggregate adjunction; Amendment U instantiated **three parallel county-keyed rungs** — CZ, MSA, state — none nesting except state→nation (`domain/dialectics/instances/levels.py`). **No `hex_adjunction` exists** — `__all__` lists only `cz_adjunction`/`msa_adjunction`, both county-as-**child**, hex-absent (`levels.py:65-77`). A county→hex rung would be **new machinery**.
- **The res-7 Postgres substrate stack** is real, live, and county:hex-1:many: `hex_hydrator.py` runs once at session init, derives per-county v/c/s/k from SQLite reference tables, allocates uniformly across each county's H3 res-7 cells, writes `dynamic_hex_state` (SPARSE delta, PK `(session_id, tick, h3_index)`, `migrations/0011`) plus an immutable `hex_spatial_map` (hex→county/state/region, `migrations/0027`, ~1.88M rows / 3,128 counties, ~602 hexes/county per `specs/105-national-canonical-acceptance/research.md:21`). Read interface is the `v_hex_state_asof` view (`migrations/0030_views_current.sql`). Wired at `postgres_initialization.py:832-868`, gated on a caller-supplied `hex_hydration_counties` set.
- **A second reference crosswalk** `bridge_county_h3` exists (`reference/schema.py:157`, query layer `engine/hydration/reference.py:99-198`) — but is **res-7** and its territory hydrator (`hydrate_territories`) is wired only into the OLD MVP facade `engine/simulation/_legacy.py:203-213`, not `_DEFAULT_SYSTEMS`.
- **A third, dead hex path:** `hex_graph_bridge.py` (R7→R6 in-memory `HexGrid` writing `hex_`-prefixed attrs onto existing Territory nodes) is gated on `services.hex_grid`, which defaults `None` everywhere in production (`engine/services.py:258,304,416`) — exercised only by `tools/demo_substrate.py` and unit tests.

> **Contradiction / open item surfaced (sweeps 2 & 4):** there are **two differently-named Postgres tables** — `dynamic_hex_state` (`migrations/0011`, has `county_fips` column, c/v/s/k + substrate stocks) and `hex_state` (`postgres_schema.py:438-452`, HEX_STATE_DDL, same `(session_id, tick, h3_index)` PK but different value columns and **no** `county_fips`, REFERENCES `hex_cell`). Their legacy-vs-current relationship was **not resolved**. Any design choosing "the Postgres hex stack" as source must resolve this first.

### 1.4 Client surfaces (legacy web)

- Live endpoint `GET /api/games/{id}/map/` → `api.game_map` → `EngineBridge.get_map_snapshot` (`engine_bridge.py:2483-2638`), zoom levels `state/bea/bea_ea/msa/county/cz/hex`.
- **1:1 hex→territory assumption is load-bearing in two places:** `h3_to_territory` dict (`engine_bridge.py:2551-2559`, scalar `h3_index`→node_id, cannot express one-node-many-hexes) and `_hex_state_row` (`:10142-10144`, returns `None` — writes zero `HexState`/`hex_latest` rows — for any territory lacking a scalar `h3_index`).
- **This is already latently broken for `us_nationwide`** (county-grain, `h3_index=None` on all 3,153 nodes): `_hex_state_row` returns `None` for every county → `HexState` never populated → `/map/` inferred to render empty at every zoom. **No test exercises `game_map` against `us_nationwide`** (only two_node/wayne_county). *Sweep 3 flags this as a strong static-analysis inference, NOT an observed runtime screenshot (see §4 blockers).*
- Frontend `DeckGLMap.tsx` shares the scalar assumption (`H3Territory` type `:96`, `hasH3` `:1150`, per-territory `h3_index` lookup `:260-287`); with no `h3_index` it degrades to the meaningless synthetic dot-grid `buildScatterFallbackLayers` (`:977-1014`). The **aggregated region path** (county/cz/msa/state via `H3ClusterLayer` keyed on `properties.member_h3`, `:474-523`) is already many-hexes-aware — but reads the same empty-if-no-`h3_index` `HexState` table.
- `SCENARIO_CATALOG` copy is already stale for `us_nationwide` ("~1,100 H3 territories", `api.py:297-302`) and would go stale for Wayne on any collapse.

### 1.5 Determinism / persistence blast radius

- **Wayne is NOT in the qa:regression byte-identical gate.** The 6 canonical scenarios are imperial_circuit ×4, two_node, `single_county` (`tools/regression_scenarios.py:24-119`). A Wayne graph-shape change **would not trip qa:regression** — but would trip the ~160-reference web/integration + 3 sentinel-registry estate (`test_contract_parity.py:237-252`, `test_dashboards.py:154-169`, `test_aw4_centrality_lens.py`, `seam/registry.py:260-373`; ~160 lines enumerated in a saved grep, ~140 unread — see §4 risk).
- **Wayne IS reached transitively by two other gates** via the Detroit tri-county headless scope (Wayne 26163 + Oakland 26125 + Macomb 26099, `scopes.py:123`): `qa:e2e-regression` / `qa:storage-budget` back `detroit-tri-county-5t.json`, `dense/detroit_tri_county.csv`, `michigan-e2e.json` (36MB), `storage-budget-5t.json` (hex-row floor `dynamic_hex_state: 209.0`). Any change touching `tests/baselines/**` requires a `Baselines: blessed(<slug>)` commit trailer (`tools/check_baseline_ceremony.py`, local hook + CI `--range` leg).
- **A structural precedent for a collapsed Wayne already exists** — `single_county.py` (Wayne FIPS 26163, one `T001` territory, `county_fips` set, `h3_index=None`, 2 entities / 3 edges) is a **separate** canonical qa:regression scenario, NOT reused by `_legacy_wayne.py` (sweeps flag the analogy as illustrative, not a ready migration path).
- **`territory_snapshot` PK is `(game_id, tick, county_fips)` with `ON CONFLICT DO NOTHING`** (`postgres_schema.py:658-710`) — an **already-lossy** collision point: for Wayne, only the first of 81 hexes to write each tick survives for heat/population (`engine_bridge.py:3602-3606`, `:3690-3694`). `node_state` (PK `(session_id, tick, node_id)`) does **not** collide.
- **No separate save-file format** — Postgres rows keyed by `session_id` ARE the durable state (`session_recorder.py:52-90`). Real hazard: **mixed-epoch sessions** (paused pre-migration sessions carry the old keying), plus every `county_fips`-keyed API (`get_territory_history`/`get_map_history` `engine_bridge.py:3599-3707`) and the CZ lookup cache (asserted exactly 3144 entries, `engine_bridge.py:2234-2261`).
- **No SQL migration exists for #39's re-keying** — it was done entirely at the Python scenario/artifact level because `county_fips` was already a generic column. `ls migrations/` shows only 0010-0035, none Wayne/#39-named.

> **Contradiction surfaced (sweep 2):** a stale docstring calls Wayne "resolution-5" while the code is `H3_RESOLUTION=6`. Any res target decision must not trust that docstring.

---

## 2. The design space

The phrase "migrate Wayne's intra-county detail onto the res-7 substrate overlay" is itself ambiguous — the sweeps show the open questions frame #57 as a *hex-substrate migration*, while the collapse-blast-radius sweeps evaluate a *collapse to county node*. These are **opposite directions**. Three candidate architectures, with the constitutional constraint that **`NodeType.HEX` on the engine graph is forbidden** (Amendment U, `CONSTITUTION.md:457`) — so no candidate may stamp hex graph nodes without an amendment.

### Candidate A — Formalize in place (keep 81 res-6 Territory nodes, add real `county_fips`)

Deepen Amendment U's lattice *downward*: keep Wayne's 81 per-hex Territory nodes exactly as-is, but stamp real `county_fips=26163` at **scenario-build time** (moving the logic out of the web bridge's `_seed_wayne_county_fips`) via a real crosswalk, and register Wayne as a sanctioned res-6 exception.

- **Preserves:** everything — 81 TENANCY edges, 4-zone class stratification (the stated design point), 3 PRESENCE-linked hexes, per-hex fog variance, the always-False bifurcation adjacency, the entire ~160-line test estate. The legacy `/map/` 1:1 machinery keeps working unchanged (nodes still carry scalar `h3_index`).
- **Breaks:** essentially nothing behaviorally. It does **not actually move Wayne onto the res-7 substrate** — it stays res-6 and stays hexes-as-nodes (surface this: it formalizes rather than migrates).
- **Constraint:** the vocabulary sentinel *forbids* deriving `county_fips` from an h3 cell variable, using Wayne's own for-loop idiom as its canonical positive test (`tests/unit/sentinels/test_node_type_vocabulary.py:643-655`). So the fips must come from a **real crosswalk**, and `bridge_county_h3` is **res-7**, not res-6 — either it must be shown to cover res-6 cells or a new res-6 artifact is needed (open question).
- **Ceremony/baseline cost:** low. No graph-shape change → qa:regression untouched; tri-county baselines likely untouched (verify). Possible vocabulary-sentinel allowlist edit.

### Candidate B — County node + res-7 substrate projection (collapse graph, relocate spatial detail)

Converge Wayne onto the USScenario/`single_county` pattern: one county-grain Territory node (`h3_index=None`, `county_fips=26163`), and relocate all intra-county spatial detail to the **already-populated res-7 Postgres stack** (`hex_spatial_map` + `dynamic_hex_state` + `v_hex_state_asof`), surfaced only through `observe()`/map-room projection — reusing spec-088's real per-hex data instead of re-deriving a fan-out.

- **Preserves:** keying uniformity (Wayne becomes the template for all counties), the county economics pipeline (no more `_seed_wayne_county_fips` collapse hack — it becomes dead code), and reuses the correct, existing res-7 crosswalk.
- **Breaks (extensive):** the 81-TENANCY-edge gameplay and 4-zone stratification are **lost** (no per-hex nodes to link classes to); `bifurcation/ceiling.py`'s adjacency check **inverts from always-False to always-True** (every class trivially shares the one node); the AW4 centrality len==3 assertion; the 65-desert/16-mud/0-water fog variance flattens to one reading; `_has_county_only_territories` (`engine_bridge.py:7627-7638`) flips Wayne into the reference-DB hex-to-county bridge code path it currently skips; and the ~160-line test estate needs rewriting. It also **inherits the latent `us_nationwide` blank-map bug** unless the write path is fixed first.
- **Ceremony/baseline cost:** high. Touches `tests/baselines/**` (single_county.json, tri-county, michigan-e2e 36MB, storage-budget) → `Baselines: blessed()` trailer + regeneration. Requires resolving `hex_state` vs `dynamic_hex_state` (§1.3). This is the direction the "collapse" sweeps measure; it is the largest blast radius.

### Candidate C — `hex_`-attribute overlay on a county node (revive `hex_graph_bridge`)

Middle path: one county-grain Territory node, but carry res-7 intra-county detail as `hex_`-prefixed attributes on that node via the existing-but-dead `hex_graph_bridge.py` (populate `services.hex_grid`, R7→R6 aggregation, `hex_graph_bridge.py:231-284`).

- **Preserves:** county-grain economics uniformity; keeps hex detail addressable without new graph node types; stays within "hex is never a graph node."
- **Breaks:** still loses the 81-node TENANCY gameplay and zone stratification (same as B — hex detail is *attributes*, not linkable nodes). Built on **currently-dead code** (`services.hex_grid` never populated in production) — reviving it is real work, and it's a **third** hex mechanism disjoint from both the Postgres stack and Wayne's current path, risking a fourth reconciliation debt.
- **Ceremony/baseline cost:** medium-high; same baseline touches as B plus dead-code revival risk.

> **What the evidence most supports:** if the goal is *gameplay-preserving formalization*, Candidate A is by far the lowest-risk and matches the "hex-substrate migration" (not "collapse") framing in `ai/state.yaml`/ADR091. If the goal is *keying unification / Wayne-as-template*, Candidate B is the honest large-blast-radius path and it should reuse the res-7 Postgres stack rather than re-derive. Candidate C is dominated by A (loses the same gameplay as B while reviving dead code).

---

## 3. Owner-decision list (BD only)

1. **Direction.** Is #57 (a) *formalize in place* — keep 81 res-6 nodes, add real `county_fips` (Candidate A), or (b) *collapse* Wayne to one county-grain node (Candidate B/C)? The ticket title says "migration" and `ai/state.yaml` says "stays hex-keyed," but the collapse sweeps assume convergence onto USScenario. **These are opposite; nothing proceeds until this is fixed.**
2. **Client target.** Does the **legacy `/map/` endpoint get the 1:many rewrite at all**, or does intra-county spatial detail land **only** in the successor terminal `observe()`/map-room projection? Amendment V/II.8 (`CONSTITUTION.md:282,459`) rules the browser client legacy and "not extended"; the TUI brief says `src/frontend/` + `web/` "idle pending a cutover decision" (`ai/_inbox/tui/20260719archiveinterfacedesign.md:125`). If the answer is "successor only," most of §1.4's blast radius becomes moot rather than something to fix.
3. **Special case vs template.** Does Wayne stay a **special-cased exception**, or does its solution become **the template for all 3,153 counties** getting res-7 intra-county detail? (Bears directly on whether to invest in a general county→hex rung.)
4. **Test estate disposition.** Is the ~160-line Wayne test/sentinel estate **load-bearing spec** (per Tests-as-Behavioral-Contracts doctrine — must be preserved/ported) or **incidental scaffolding** (rewrite wholesale)? This gates the true cost of B/C.
5. **Resolution.** Res-6 (Wayne today) or res-7 (constitutional "immutable substrate")? Resolve the stale "resolution-5" docstring in the same ruling.
6. **Map-room tier (TUI §10 open question).** Does the successor map-room commit to **any** hex-grain choropleth tier, or stop at county as its finest tier? (`ai/_inbox/tui/20260719archiveinterfacedesign.md:144` — still open.) If county is the finest successor tier, a hex-level 1:many migration matters far less to the successor than to any lingering legacy web work.
7. **Crosswalk artifact.** If per-hex `county_fips` is needed: does the res-7 `bridge_county_h3` cover Wayne's res-6 cells, or is a new res-6 crosswalk artifact required? (Sweep open question; the vocabulary sentinel forbids h3→fips derivation.)

---

## 4. Blockers, prerequisites, and recommended sequencing

### Blockers / prerequisites

- **B1 — Unverified blank-map inference.** The `us_nationwide` "renders empty" claim is static-analysis inference (the `_hex_state_row` `None`-gate + empty-table read chain), **not an observed runtime result** (sweep 3 explicit). Confirm/deny by loading a live `us_nationwide` session map *before* choosing any collapse path — else Wayne silently inherits the same defect (Candidate B/C).
- **B2 — Two Postgres hex tables unresolved.** `hex_state` vs `dynamic_hex_state` legacy-vs-current status (§1.3) must be resolved before any design names "the Postgres hex stack" as the projection source.
- **B3 — Vocabulary sentinel + crosswalk resolution.** Any per-hex `county_fips` stamping needs a real crosswalk (not h3-derived) and possibly a res-6 artifact + allowlist edit (`UNSTAMPED_QUERY_ALLOWLIST`).
- **B4 — Baseline ceremony.** Any `tests/baselines/**` touch (unavoidable for B/C: single_county, tri-county, michigan-e2e 36MB, storage-budget) requires a well-formed `Baselines: blessed(<slug>)` trailer surviving squash/rebase, or CI `--range` rejects it.
- **B5 — Runtime cell-count verification.** The "81 cells" figure rests on a code comment + test assertions, never a runtime `polygon_to_cells` execution (sweep risk) — h3 version drift could shift it silently.
- **B6 — TUI §10 unresolved (decision #6).** No ratified successor-side hex-grain rendering target exists; "design it as an `observe()` projection" is directionally correct per Amendment V but has no concrete shape until the BD rules the tier question.
- **B7 — Mixed-epoch sessions.** No save-file format; paused pre-migration Postgres sessions carry the old keying. Any re-keying needs a stance on resume/replay of pre-existing sessions (backfill vs invalidate).

### Recommended sequencing

1. **BD answers decisions #1 and #2 first** (direction + client target). These bisect the entire design space; everything downstream is contingent.
2. **Discharge B1 + B2 + B5** (cheap, read-only/runtime verification: load the `us_nationwide` map; trace live writers for `hex_state`; run `polygon_to_cells` once). These are prerequisites regardless of direction and de-risk the collapse path.
3. **BD answers #4** (test estate = spec or scaffolding) — this sets the true cost basis for B/C vs A.
4. **If Candidate A** (formalize in place): resolve B3 (crosswalk/res + sentinel), move `county_fips` stamping from the web bridge into the scenario, register the sanctioned exception. No baseline ceremony expected — verify tri-county baselines unmoved.
5. **If Candidate B** (collapse + projection): fix the write path (B1 defect) *before* collapsing; build the `observe()` projection reading `v_hex_state_asof` per successor R5; **do not extend legacy `/map/`** unless decision #2 says so; regenerate baselines under B4 ceremony.
6. **Defer any general county→hex `ScaleAdjunction` rung** until decision #3 (special-case vs template) — building new lattice machinery is unjustified for a single special-cased county.
