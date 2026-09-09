# SubstrateSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `SubstrateSystem` (@2.5, Material Base) is a small (191
logical lines, `src/babylon/engine/systems/substrate.py`), two-part system: (1) a
per-territory `raw_material_stock` depletion rule that is the SAME formula family
(`calculate_biocapacity_delta`) MetabolismSystem already has a landed BSL pack
for, and (2) a scale-lattice aggregate (county → commuting-zone/MSA/state/nation
sums) published into `context.persistent_data`, the first and so far only engine
consumer of `ScaleAdjunction`. Part 1 is portable now on Metabolism's own
precedent, modulo one new content-modeling D-record (no BSL string-identity or
field-presence test exists for the eligibility guard). Part 2 is not
rule-content at all under the current BSL surface — it needs graph-scope
multi-key aggregate storage nothing in the language serves yet, and it is a
dead channel with zero production consumers and zero exercise on any canonical
`qa:regression` scenario today.

**Verdict:** SPLIT DISPOSITION — the per-territory depletion rule is PORTABLE
WITH D-RECORDS (Metabolism's D-1 precedent covers the coefficient-domain
hazard directly); the eligibility guard needs a new D-record (no BSL
string/presence test); the scale-lattice binding + aggregate-publish half is
NOT-A-PACK — it is substrate/engine infrastructure, blocked on graph-scope
multi-key storage BSL has no home for, and today a dead channel dormant on
every canonical scenario.

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/substrate.py` | 339 | **The target.** `SubstrateSystem`, `step()` at 193-260. |
| `src/babylon/config/defines/substrate.py` | 60 | `SubstrateDefines` Pydantic model — `depletion_scale`/`regeneration_rate`/`entropy_factor`. |
| `src/babylon/data/defines.yaml` | (substrate block: lines 1017-1020) | Player-editable coefficient values, `substrate:` section. |
| `src/babylon/config/defines/_assembler.py` | (line 219) | `GameDefines.substrate: SubstrateDefines` — the field `services.defines.substrate` resolves. |
| `src/babylon/models/entities/territory.py` | 352 | `Territory` entity — `county_fips` (81-91), `extraction_intensity` (171-176), `raw_material_stock` (181-194), `raw_material_capacity` (195-209). Shared file with TerritorySystem's own field set; SubstrateSystem touches only these four. |
| `src/babylon/models/enums/topology.py` | (line 61) | `NodeType.TERRITORY = "territory"` — the only node type queried. |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase._read` (119-159, `required=True` diagnostics), `._write_clamped` (161-191, `max(lo, min(hi, value))`), `._wrap_graph` (98-117). |
| `src/babylon/kernel/graph_protocol.py` | (relevant: 77-88 `update_node`, 258-274 `query_nodes`) | `GraphProtocol` signatures. |
| `src/babylon/topology/graph.py` | (relevant: 651-669) | Concrete `BabylonGraph.update_node`/`get_node` — plain dict merge, **no type coercion or quantization at tick time**. |
| `src/babylon/kernel/tick_partition.py` | 31 | `TickPartition.MATERIAL_BASE`. |
| `src/babylon/formulas/metabolic_rift.py` | 119 | **`calculate_biocapacity_delta` ONLY** (lines 9-53) — the system's sole formula call. `calculate_hysteresis_damage`/`calculate_overshoot_ratio` (55-119) live in the same module but are NEVER called by `SubstrateSystem` (grep-confirmed). |
| `src/babylon/domain/economics/tick/graph_bridge.py` | 586 | `resolve_county_identity` (44-77) — reads `Territory.county_fips`, returns `None` for an abstract territory. Shared with `vol2_circulation.py:66,203` (not Substrate-exclusive). |
| `src/babylon/domain/dialectics/instances/scale.py` | 177 | `ScaleAdjunction` — `mapping`/`shares`, `.aggregate()` (141-156, EXTENSIVE sum), `.aggregate_intensive()` (158-177, unused here), `.uniform()` (103-118). Fully read. |
| `src/babylon/domain/dialectics/instances/levels.py` | 651 | `spatial_lattice_rungs_for_counties` (514-588), `cz_adjunction` (367-409, committed CSV), `msa_adjunction` (444-473, opens a reference-DB session), `SpatialLatticeRungs` dataclass (476-511). Fully read. |
| `src/babylon/kernel/services.py` | (relevant: 23-40) | `ServicesProtocol.defines: Any` — resolves to `GameDefines` at runtime. |
| `src/babylon/engine/context.py` | (relevant: 27-98) | `TickContext.persistent_data: dict[str, Any]` — the write target for all four aggregates + the exclusion list. |
| `src/babylon/engine/simulation_engine.py` | (relevant: 328-363) | `_SYSTEM_CLASSES` tuple — confirms tick position 2.5, between `TerritorySystem` (2.0) and `ProductionSystem` (3.0). |
| `src/babylon/engine/systems/production.py` | (relevant: 251-268) | Writes `extraction_intensity` (@3.0, AFTER Substrate) — the one-tick-lag upstream source. |
| `src/babylon/engine/scenarios/_legacy.py` | (relevant: 702-828) | `_create_us_territories` — the **only** production code path that ever seeds `raw_material_stock`/`raw_material_capacity` (lines 800-826), from `us_county_territories.json`'s `raw_material_value_millions`. Feeds `create_us_scenario` (the `us_nationwide` campaign, aliased in `web/game/engine_bridge.py:7493`) — **not** a member of `tools/regression_scenarios.py`'s `SCENARIOS` dict (§5). |

**Not exercised by `substrate.py` at all, despite sharing the word "substrate"
— two genuine namespace collisions, verified by reading the imports of every
candidate file:**

- `src/babylon/domain/economics/substrate/` (14 modules — `aggregation.py`,
  `circulation.py`, `conservation.py`, `equalization.py`, `ground_rent.py`,
  `hydrator.py`, `production.py`, `spatial.py`, `transitions.py`, `types.py`,
  …) — the **tri-county H3-mesh Vol I/II/III economic substrate** (feature
  026-tri-county-economic-substrate, `__init__.py:1-8`: "Integrates Capital
  Volumes I/II/III... onto an H3 resolution 7 spatial mesh covering
  Wayne/Oakland/Macomb counties"). Wholly unrelated code, unrelated formulas,
  unrelated tests (`tests/unit/economics/substrate/`, 14 files, 4199 lines;
  `tests/integration/economics/test_substrate_pipeline.py`, 361 lines).
  `SubstrateSystem` imports nothing from this package (grep-confirmed).
- `src/babylon/persistence/hex_hydrator.py` / `hex_state.py` / the
  `dynamic_hex_state` Postgres table — a **different** `raw_material_stock`
  column, on the per-hex persistence layer (spec-066, apportioned by
  population/area weighting, `hex_hydrator.py:240-261`), read by the
  `babylon.engine.headless_runner` pipeline (`qa:e2e-regression`'s
  `detroit-tri-county` scope), which is a **wholly separate execution path**
  from `SimulationEngine`/`_DEFAULT_SYSTEMS` (`headless_runner` does not
  invoke `SubstrateSystem`). `tests/unit/persistence/test_substrate_apportionment.py`
  (176 lines) tests this column, not `SubstrateSystem`.

**Reference BSL pack read for format** (fully read):

- `rust/crates/babylon-tick/content/rules/metabolism.bsl` (413 lines) — the
  SAME `calculate_biocapacity_delta` formula, already ported as
  `metabolism/biocapacity-update`. Its D-1 finding (`entropy_factor`'s
  `(1.0, 3.0]` domain has no legal `Real × Ratio` operator, scaled-bare-Int
  workaround) transfers directly to `SubstrateSystem`'s `entropy_factor` (same
  domain, same default `1.2`) and `depletion_scale` (`[0.0, 10.0]`, same
  operator-mismatch shape). Its D-2 (provably-uniform `:const` reasoning), D-4
  (`(domain :graph)` unserved-at-execution finding) and D-5 (silent-default vs.
  loud-failure transcription choice) are all directly load-bearing precedent
  for this system too (§2, §4, §6).

## 2. COMPUTATION CATALOG (execution order, `substrate.py:207-260`)

### Step 1 — Eligibility filter (`substrate.py:209-219`)

- **(a)** Only `TERRITORY` nodes carrying BOTH a real `county_fips` AND a
  non-`None` `raw_material_stock` are touched; everything else (abstract
  territories, or a county with no `fact_state_minerals`/geometry row) is
  skipped forever — "never a fabricated default" (`substrate.py:27-29`).
- **(b)** `resolve_county_identity(node) is not None and
  node.attributes.get("raw_material_stock") is not None`
  (`substrate.py:213-214`), sorted by node id (`:216`) for deterministic
  iteration order.
- **(c) Reads:** `TERRITORY.county_fips` (via `resolve_county_identity`,
  `graph_bridge.py:73-77`), `TERRITORY.raw_material_stock` presence.
- **(d) Writes:** none (a pure filter).
- **(e) Defines:** none.
- **(f) Events:** none.

### Step 2 — Per-territory depletion (`substrate.py:221-245`, loop over eligible)

- **(a)** Each eligible territory's raw-material stock regenerates toward its
  (immutable, per-node) ceiling and is depleted by extraction at an entropic
  loss — the SAME formula MetabolismSystem's `biocapacity` uses, applied to a
  PARALLEL dollar-denominated mineral-value stock with its own coefficients
  (module docstring, `substrate.py:31-38`).
- **(b)** `delta = calculate_biocapacity_delta(regeneration_rate=defines.regeneration_rate,
  max_biocapacity=ceiling, extraction_intensity=extraction_intensity *
  defines.depletion_scale, current_biocapacity=current_stock,
  entropy_factor=defines.entropy_factor)` (`substrate.py:230-236`), where
  inside the formula (`metabolic_rift.py:39-53`):
  - `regeneration = regeneration_rate * max_biocapacity` (`:40`); forced to
    `0.0` if `current_biocapacity >= max_biocapacity` (`:43-44`).
  - `raw_extraction = extraction_intensity * current_biocapacity` (`:47`) —
    the `extraction_intensity` operand here is ALREADY the
    `defines.depletion_scale`-scaled value from the call site, so this is
    really two chained multiplies: `depletion_scale × extraction_intensity ×
    current_stock`.
  - `ecological_cost = raw_extraction * entropy_factor` (`:50`).
  - `delta = regeneration - ecological_cost` (`:52`).
  Then `new_stock = self._write_clamped(protocol, node.id,
  "raw_material_stock", current_stock + delta, lo=0.0, hi=ceiling)`
  (`substrate.py:237-244`), where `ceiling = self._read_ceiling(node)`
  (`:228`, `:262-292`) — `raw_material_capacity`, read fresh every tick, never
  mutated by this system (unlike Metabolism's ratcheting `max_biocapacity`).
- **(c) Reads:** `TERRITORY.raw_material_stock` (pre-tick, `required=True`,
  `:227`), `TERRITORY.raw_material_capacity` (`required=True`, `:283`, raises
  `KeyError`/`ValueError` on absence — a genuine seeding-bug signal, not an
  honest gap, per the docstring at `:264-291`), `TERRITORY.extraction_intensity`
  (last tick's `ProductionSystem` write; **read via bare
  `node.attributes.get("extraction_intensity", 0.0)`, `:229` — NOT
  `self._read(..., required=True)`**, unlike the other two reads on this same
  node — a silent-default read pattern, inconsistent with this system's own
  convention two lines earlier).
- **(d) Writes:** `TERRITORY.raw_material_stock`, clamped `[0,
  raw_material_capacity]`.
- **(e) Defines:** `substrate.depletion_scale` (1.0, `[0.0, 10.0]`),
  `substrate.regeneration_rate` (0.0, `[0.0, 1.0]`), `substrate.entropy_factor`
  (1.2, `(1.0, 3.0]`) — `defines.yaml:1018-1020`,
  `config/defines/substrate.py:24-57`. **All three are GLOBAL `SubstrateDefines`
  coefficients, never per-node `Territory` fields** — unlike Metabolism's
  `regeneration_rate`, which is a per-node `Territory` field that merely
  happens to be provably uniform (metabolism.bsl D-2); Substrate's
  `regeneration_rate` was never per-node to begin with, a strictly simpler
  provenance.
- **(f) Events:** none.

### Step 3 — Lattice build, first eligible tick only (`substrate.py:247-250, 294-328`)

- **(a)** On the FIRST tick with ≥1 eligible territory, build the four
  Amendment U scale rungs (commuting zone / MSA / state / nation) over the
  eligible county set and cache them for the life of the `SubstrateSystem`
  instance — never rebuilt (module docstring "Lattice binding", `:46-57`;
  assumes a FIXED county universe for the run, #39 T6 LOW-3).
- **(b)** `rungs = spatial_lattice_rungs_for_counties(sorted(stock_by_county),
  cz_adjunction_fn=self._cz_adjunction_fn, msa_adjunction_fn=self._msa_adjunction_fn)`
  (`levels.py:514-588`): `state`/`nation` are TOTAL over every requested county
  (`_state_parent_map`/`_nation_parent_map`, `levels.py:265-282`); `cz` is
  restricted to counties the committed 1990 ERS crosswalk covers, the rest
  named on `cz_excluded` (`levels.py:565-571`); `msa` is silently partial
  (`:573-576`). This is Python/engine-instance object state (`self._rungs`),
  not per-tick math — see §6.
- **(c) Reads:** the eligible county-fips set (from Step 1/2's output); the
  committed `bridge_county_cz.csv` (`levels.py:88-92`); a reference-DB session
  for `msa_adjunction()` (`levels.py:469-473`, opened once, ever, per
  instance — never for the canonical scenarios, §5).
- **(d) Writes:** `self._rungs` (System-instance memory, not the graph).
- **(e) Defines:** none.
- **(f) Events:** none (a `logger.warning` on nonempty `cz_excluded`,
  `substrate.py:322-327` — not an `EventType`).

### Step 4 — Extensive aggregate publish (`substrate.py:252-257`)

- **(a)** Every tick (after the lattice is built), sum the post-depletion
  `raw_material_stock` across the four rungs and publish the results plus the
  CZ-exclusion list into `context.persistent_data`, "mirroring
  `SovereigntySystem`'s mechanism" (`substrate.py:60-61`).
- **(b)** `persistent[SUBSTRATE_CZ_KEY] = rungs.cz.aggregate(stock_by_county)`;
  same for `MSA`/`STATE`/`NATION` (`substrate.py:253-256`); each `.aggregate()`
  call is `scale.py:141-156`: `result[parent] = Σ by_child[child]` for every
  `child → parent` in the rung's mapping — EXTENSIVE (summed), never
  `aggregate_intensive` (module docstring `:58-60` is explicit about this).
  `persistent[SUBSTRATE_CZ_EXCLUDED_KEY] = list(rungs.cz_excluded)`
  (`:257`, sorted tuple → list).
- **(c) Reads:** `stock_by_county` (this tick's own Step-2 output, in-memory —
  not re-read off the graph).
- **(d) Writes:** `context.persistent_data["substrate.cz"]`,
  `["substrate.msa"]`, `["substrate.state"]`, `["substrate.nation"]` (each
  `dict[str, float]`), `["substrate.cz_excluded"]` (`list[str]`) —
  `SUBSTRATE_CZ_KEY` et al., `substrate.py:141-145`.
- **(e) Defines:** none.
- **(f) Events:** none.

**Events emitted by the whole system: zero.** Grep-confirmed — no `EventType`,
`emit`, `.publish(`, or `_publish` reference anywhere in `substrate.py`.
Matches `TerritorySystem`'s own silence on the event bus.

## 3. TYPE INVENTORY

Runtime storage note (load-bearing, identical to Territory's own finding):
`BabylonGraph.update_node` (`topology/graph.py:660-669`) is a plain dict merge
with no type coercion or quantization. `Territory`'s `SnapToGrid` (1e-5 grid)
applies **only when the Pydantic model is instantiated** (scenario seed /
`WorldState` round-trip) — but see below, three of SubstrateSystem's four
touched fields don't even carry `SnapToGrid` at instantiation time, because
they are declared as plain `float`, not the `Currency`/`Intensity` Annotated
aliases.

| Attribute | Node type | Python model type | Domain | Category |
|---|---|---|---|---|
| `county_fips` | TERRITORY | `str \| None` | 5-char FIPS, `min_length=5, max_length=5` (`territory.py:81-91`) | **string identity, read-only here** |
| `raw_material_stock` | TERRITORY | `float \| None` (plain, `Field(ge=0.0)`, **not** the `Currency` Annotated alias — `territory.py:181-194`) | `[0.0, ∞)`, no upper Pydantic bound (clamped in-tick to `[0, raw_material_capacity]`) | **unbounded real, nullable sentinel — NO `SnapToGrid` at any stage** |
| `raw_material_capacity` | TERRITORY | `float \| None` (plain `Field(ge=0.0)`, same non-Currency shape — `:195-209`) | `[0.0, ∞)` | **unbounded real, nullable sentinel, immutable after seed — read-only here** |
| `extraction_intensity` | TERRITORY | `float` (plain `Field(ge=0.0, le=1.0)`, **not** the `Intensity` Annotated alias — `:171-176`) | `[0.0, 1.0]` | unit-interval, read-only here, **written by a different System (Production)** |
| `depletion_scale` (define) | — | `float` | `[0.0, 10.0]` | **out-of-`[0,1]` real coefficient** |
| `regeneration_rate` (define) | — | `float` | `[0.0, 1.0]` | unit-interval coefficient |
| `entropy_factor` (define) | — | `float` | `(1.0, 3.0]` | **out-of-`[0,1]` real coefficient** |
| `substrate.cz`/`.msa`/`.state`/`.nation` | — (persistent_data, not a node attribute) | `dict[str, float]` | keyed by arbitrary CZ-id/CBSA-code/2-digit-state/`"US"` strings | **string-keyed aggregate map — no node/attribute shape at all** |
| `substrate.cz_excluded` | — (persistent_data) | `list[str]` | sorted county-FIPS subset | **string list — no node/attribute shape at all** |

**Zero enum discriminants, zero bools.** Unlike `TerritorySystem`,
`SubstrateSystem` reads/writes no `StrEnum`-typed field and no `bool`-typed
graph attribute at all — a structurally favorable difference (no
`deffield enum` content-modeling question).

**No `Currency`/`Intensity` type flag — genuinely favorable, opposite of
Territory's finding.** Territory's `rent_level` is the `Currency` Annotated
alias (SnapToGrid-quantized at instantiation, ge=0.0 domain identical to
BSL's own `Currency` semantics in spirit). `raw_material_stock`/
`raw_material_capacity` are declared as **plain, un-annotated `float`
fields** — they never carry `SnapToGrid` quantization even at scenario-seed
time, and — because `GraphSubstrate` attribute storage is always plain `f64`
and `scenario.rs` refuses `Currency`-typed field STORAGE outright
(`scenario.rs:1067,1158,1244`, confirmed against dev) — a port declaring them
`int extensive` (the landed-pack precedent for every other money-like field:
`metabolism.bsl`'s `territory/biocapacity`, `territory/max-biocapacity`) hits
**no** Currency-vs-Real tension at all, because the Python type was never
`Currency`-flavored to begin with. `extraction_intensity` is likewise a plain
`float`, not `Intensity` — same favorable non-issue.

**String-identity flag — a genuine, previously-unnamed gap (verified live
against `docs/reference/bsl-language.rst`, current dev).** `deffield`'s
closed type vocabulary is `{int, bool, currency, probability, intensity,
coefficient, enum}` (bsl-language.rst §3.1, `declarations.rs`
`parse_deffield`) — there is no string/text row. `Str` exists ONLY as a
typechecker classification "at `:material-basis` and in vector ids"
(bsl-language.rst:2394-2398, read directly) — it is explicitly named as one
of the types "no `<type-name>` position can name." `county_fips` (a 5-char
FIPS string) has **no direct BSL field-storage representation**, and neither
does the `substrate.cz`/etc. string-keyed dict output (§6).

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `float`); **zero libm
transcendentals anywhere in the call chain** — grep-confirmed zero
`exp`/`log`/`sigmoid`/`pow` in `substrate.py` AND in
`metabolic_rift.py::calculate_biocapacity_delta` (read in full). Shapes, in
execution order:

1. **Presence checks (not arithmetic):** `is not None` ×2 (`substrate.py:213-214`).
2. **Widening casts, no precision loss:** `float(...)` ×3 (`:227, 229, 292`) —
   no truncation, unlike a Real→Int demotion.
3. **Multiplicative scale (chained):** `extraction_intensity * defines.depletion_scale`
   (`substrate.py:233`, the call-site pre-scale) then, inside the formula,
   `raw_extraction = extraction_intensity * current_biocapacity`
   (`metabolic_rift.py:47`) — effectively THREE operands multiplied
   (`depletion_scale × extraction_intensity × current_stock`), not two.
4. **Regeneration multiply + branch:** `regeneration_rate * max_biocapacity`
   (`:40`); `if current_biocapacity >= max_biocapacity: regeneration = 0.0`
   (`:43-44`) — **the bare `0.0` literal is the SAME "no bare non-integer
   literal" BSL parser problem Territory/Vitality/Metabolism already carry a
   documented idiom for**; `metabolism.bsl` already expresses this EXACT
   branch of this EXACT formula with the Real-zero-promotion trick,
   `(- 0 0c)` (`metabolism.bsl:365-366`) — directly reusable, not a new
   derivation.
5. **Entropy multiply (flagged hazard):** `ecological_cost = raw_extraction *
   entropy_factor` (`:50`) — `entropy_factor`'s declared domain `(1.0, 3.0]`
   is out-of-`[0,1]`, the **same D-1-class hazard** Metabolism's own
   `entropy_factor` hits, confirmed LIVE against current dev
   `bsl-language.rst:2424-2433` ("`Currency + Real`, `Currency × Currency`,
   and `Currency × Int` are type errors... An UNDECLARED coefficient greater
   than `1`... stays exactly `E-TYPE-030`") and `:2482-2484` (`Ratio`'s only
   legal operator is `Currency × Ratio`; a `Real × Ratio` — this system's
   actual shape, since `raw_extraction` is never `Currency` — has no operator
   at all). `depletion_scale` (`[0.0, 10.0]`) hits the **identical** shape for
   the identical reason (item 3's multiply is also `Real × Real`, never
   `Currency`-involving).
6. **Subtract:** `delta = regeneration - ecological_cost` (`:52`).
7. **Add:** `new_stock = current_stock + delta` (`substrate.py:241`, inline at
   the `_write_clamped` call).
8. **Clamp:** `max(lo, min(hi, value))` (`system_base.py:189`) with `lo=0.0`,
   `hi=ceiling` (a PER-NODE, non-constant bound — `raw_material_capacity`).
   **Structurally simpler than Metabolism's own double clamp**: Metabolism's
   ceiling itself ratchets down every tick (hysteresis damage), requiring TWO
   nested clamps (`new_max` floor, then `current+delta` capped at `new_max`
   then floored again); Substrate's ceiling never changes, so ONE clamp call
   with both bounds suffices — a single nested-`if` in BSL terms (no scalar
   `min`/`max` in the grammar, matching every landed pack's convention).
9. **Extensive summation (Step 4):** `result[parent] += by_child[child]`
   (`scale.py:155`) for every `child → parent` pair — plain accumulation, no
   transcendental. **Deterministic iteration order**: `self.mapping` is built
   from `sorted(set(counties))` at every call site (`levels.py:565, 569, 573,
   578-579`), so summation order is FIPS-sorted, not insertion- or
   hash-order-dependent — a port must replicate this exact sorted order to
   stay bit-reproducible, since float summation is not associativity-invariant.

**Real→Int demotions: zero.** Grep-confirmed no `int(...)` call anywhere in
`substrate.py` — genuinely cleaner than both Territory (2 demotions) and the
Python reference's own broader estate; no `floor` intrinsic is even needed
for this system.

**Clamp implementations: one, applied consistently.** Unlike Territory's
two-clamp inconsistency (`_write_clamped` vs. a hand-written `min(1.0, ...)`),
`SubstrateSystem` uses `_write_clamped` for its only clamped write — no
inconsistency to transcribe.

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 2.5** (`substrate.py:162`), confirmed against
  `_SYSTEM_CLASSES` (`simulation_engine.py:328-363`): `VitalitySystem (1.0) →
  TerritorySystem (2.0) → SubstrateSystem (2.5) → ProductionSystem (3.0) →
  ...`.
- **Reads from a same-tick prior system: none.** `TerritorySystem` (2.0,
  immediately prior) writes `heat`/`rent_level`/`population`/`under_eviction`
  — grep-confirmed no overlap with what Substrate reads. The
  `extraction_intensity` Substrate reads is `ProductionSystem`'s write from
  the **PREVIOUS** tick (Production runs at 3.0, after Substrate, within the
  same tick — a genuine one-tick lag by pipeline position, `substrate.py:40-44`,
  tested by `TestOneTickLag` in `test_substrate.py`).
- **Writes consumed downstream: none.** `TERRITORY.raw_material_stock` and
  `.raw_material_capacity` are grep-confirmed **read by no other System**
  anywhere in `src/babylon/engine/systems/` — a pure terminal/observational
  output, unlike Territory's `population` (13 downstream readers). The four
  `context.persistent_data["substrate.*"]` keys are likewise grep-confirmed
  read by **no production code anywhere** in `src/babylon/` outside this
  System's own writes — a genuinely dead channel today, explicitly forward-
  documented as "Public for downstream consumers (**#39 T7**)"
  (`substrate.py:140`), a task that has not landed.
- **A shared-input, not a hidden coupling:** `extraction_intensity` is also
  read by `MetabolismSystem` (`metabolism.py:98,109`) into a COMPLETELY
  SEPARATE stock family (`biocapacity`, not `raw_material_stock`) — the
  module docstring is explicit that these are "genuinely PARALLEL"
  applications of the same formula shape over different physical quantities
  (`substrate.py:31-38`), never touching each other's output.
- **`county_fips` is a shared county-identity surface, not Substrate-exclusive:**
  `resolve_county_identity` (`graph_bridge.py:44-77`) is also imported by
  `vol2_circulation.py:66,203`, and `single_county.py`'s own docstring names
  `TickDynamicsSystem` as the field's other production consumer
  (`single_county.py:10-14`) — the string-identity BSL gap (§3, §6) is a
  shared blocker across the whole county-grain economics estate, not unique
  to this system.
- **Context/service usage with no BSL equivalent:** `context.persistent_data`
  (WRITE-only here — Substrate reads none of it back) is the same Q6
  "graph-scope state" gap class Metabolism's D-3 names for
  `balkanization.metabolic_impact_by_territory`; `services.defines.substrate`
  (`GameDefines.substrate: SubstrateDefines`, `_assembler.py:219`) is an
  ordinary defines lookup, unproblematic — the same pattern every landed pack
  uses via `defconst`.
- **DORMANCY on canonical scenarios — confirmed by direct enumeration, not
  inference.** `tools/regression_scenarios.py`'s `SCENARIOS` dict currently
  registers **12** scenarios (counted by parsing the dict literal directly:
  `imperial_circuit, two_node, starvation, glut, fascist_bifurcation,
  single_county, mitterrand, syriza, weimar, debs, bernie_valve, org_probe`
  — the "5 canonical qa:regression scenarios" language in `substrate.py`'s own
  module docstring and in `test_substrate.py`'s `TestCanonicalNoOp` class
  docstring is STALE relative to current dev, though its conclusion still
  holds). **Six** of the twelve DO stamp `county_fips="26163"` on their sole
  territory (`single_county.py:79,92,116`; `electoral_goldens.py:332,424` via
  `mitterrand`/`syriza`/`weimar`/`debs`/`bernie_valve`, all built on
  `create_two_node_scenario()` + a `model_copy` adding `county_fips`) — but
  **none of the twelve** ever sets `raw_material_stock`/`raw_material_capacity`
  (grep-confirmed: those two fields are set in exactly one production
  function anywhere in `src/babylon/engine/scenarios/`, `_create_us_territories`,
  `_legacy.py:825-826`, which is NOT one of the 12 factories `SCENARIOS`
  wires). **Every one of the 12 canonical `qa:regression` scenarios is
  therefore a structural no-op for `SubstrateSystem`** — both Step 2
  (depletion) and Step 3/4 (lattice binding + aggregate publish) never fire.
  The only production seeder, `create_us_scenario` (the `us_nationwide`
  campaign, aliased `web/game/engine_bridge.py:7493-7495`), is reached
  through the legacy web bridge, not through any byte-identical gate this
  repo runs (`qa:regression`'s `SCENARIOS`, and separately
  `qa:e2e-regression`, which drives a wholly different execution path —
  `babylon.engine.headless_runner` against the unrelated H3-mesh
  `domain.economics.substrate` module, §1 — neither touches
  `SimulationEngine`/`_DEFAULT_SYSTEMS` with a county-seeded `Territory`
  carrying `raw_material_stock`). `tests/integration/test_substrate_pipeline_position.py`'s
  own docstring confirms this directly from the live-Postgres integration
  fixture's perspective: "The hex-node graphs below therefore now exercise
  the NO-OP path (zero eligible Territory nodes...)". **A port's conformance
  fixtures must be entirely hand-built** (`.bscn`, matching Metabolism's own
  precedent) — no canonical scenario, hand-built or otherwise in the current
  estate, provides a usable seed.

## 6. BLOCKER ASSESSMENT

Adjudicated against the CURRENT BSL surface (query-lane Slice 1 landed,
ADR197; enum fields landed, ADR195/196; `floor` intrinsic landed, PR #489;
all verified live against `rust/crates/babylon-bsl/src/` and
`docs/reference/bsl-language.rst` on dev, not cited from memory).

| Computation | Verdict | Detail |
|---|---|---|
| Eligibility guard: `county_fips is not None AND raw_material_stock is not None` (`substrate.py:213-214`) | **BLOCKED — no string-identity field type / no field-presence test** | `deffield`'s closed vocabulary (`{int, bool, currency, probability, intensity, coefficient, enum}`) has no string row; `Str` is typechecker-only, confined to `:material-basis`/vector ids (`bsl-language.rst:2394-2398`, read directly). Separately, `:optional` REQUIRES `:default` (a bare `:optional` is `E-PARSE-031`) and **"there is consequently no `bound?` predicate in the language"** (`bsl-language.rst:2628-2633`, read directly) — there is no way to ask "was this field ever set" at all, only "substitute a default when absent." **Workaround-with-D-record available, not resolved as written**: a seed-time boolean discriminator field (e.g. `territory/substrate-eligible`) folding both frozen-code conditions into one `bool` — the same class of content-modeling deviation as Territory's enum→bool workaround — would make the GUARD portable; the field's own present-but-null semantics remain unrepresentable regardless. |
| Per-territory depletion math (`calculate_biocapacity_delta`, `substrate.py:221-245` + `metabolic_rift.py:39-53`) | **PORTABLE WITH D-RECORD** | Direct structural AND formula match to the ALREADY-LANDED `metabolism/biocapacity-update` rule (`metabolism.bsl:347-412`). `depletion_scale`/`entropy_factor` (both out-of-`[0,1]`) need the SAME D-1-class scaled-bare-Int workaround Metabolism's `entropy_factor` already proved out — confirmed live against dev, not inherited by citation alone (§4 item 5). `regeneration_rate` is a plain `c`-suffixed `Coefficient` `defconst`, simpler than Metabolism's own D-2 (never per-node in Python to start with — no "provably uniform" argument even needed). The bare `0.0` ceiling-branch literal reuses `metabolism.bsl`'s own Real-zero-promotion idiom verbatim (`(- 0 0c)`). The clamp is a single nested-`if` (simpler than Metabolism's double clamp — this system's ceiling never ratchets). |
| `raw_material_capacity` read (the ceiling, `substrate.py:228,262-292`) | **PORTABLE WITH D-RECORD** | An ordinary `:field territory/raw-material-capacity` read, `int extensive`-declared per the landed-pack Currency-avoidance precedent (§3) — no operator-domain issue at all (used only as a clamp bound, never multiplied). The frozen code's loud `KeyError`/`ValueError` on an absent/`None` ceiling on an otherwise-eligible node (`substrate.py:274-291`) is a defect-shaped edge case worth transcribing port-as-is (a required-field-missing failure mode, not a data gap) — but it is UNREACHABLE if the eligibility-guard D-record above folds ceiling-presence into the same discriminator field. |
| `extraction_intensity` read (`substrate.py:229`) | **PORTABLE WITH D-RECORD** | An ordinary `[0,1]`-domain `:field` read — trivial. **Transcription-decision flag, port-as-is**: the frozen code reads this via a bare `.attributes.get("extraction_intensity", 0.0)` (silent default on absence), NOT the `required=True` pattern the SAME loop uses for `raw_material_stock`/`raw_material_capacity` two lines apart — the same silent-default-vs-loud-failure choice Metabolism's own D-5 already named and deliberately did NOT carry forward (III.11 "Loud Failure"); this port faces the identical choice for this one field, unresolved by precedent since Metabolism ported ALL FOUR of its reads the loud way. |
| Lattice build (`spatial_lattice_rungs_for_counties`/`ScaleAdjunction` construction, `substrate.py:247-250,294-328`) | **NOT-A-PACK — substrate/engine infrastructure, not rule content** | Python/engine-INSTANCE object state (`self._rungs`, built once, cached for the instance's lifetime, sourced from a committed CSV plus a reference-DB session) — there is no per-tick, per-node BSL rule shape here at all; it precedes and is orthogonal to anything a `(rule ...)` form expresses. Matches the standing "spatial adjacency = static lookup estate" ruling (invariant substrate → per-resolution static lookup tables, never per-tick state; Rust side = CSR at startup) precisely: county→CZ/MSA/state/nation membership is exactly this shape. The correct disposition is native `babylon-graph`/engine startup infrastructure, never BSL content. |
| Extensive aggregate publish (`ScaleAdjunction.aggregate` ×4 into `context.persistent_data`, `substrate.py:252-257`) | **BLOCKED — graph-scope multi-key aggregate storage; name the exact missing lane** | Three compounding gaps, each verified against current dev: **(a)** `(domain :graph)` parses and typechecks at load time (`domain.rs:37-243`) but is UNSERVED at execution — `babylon-bsl/src/tick.rs::run_tick` (524-538) derives its subject type SOLELY from `subject_type_of(&loaded.bindings)` (a `:field`-binding scan) and never reads `loaded.domain` anywhere in the function body (confirmed by reading the full function) — the same class Metabolism's D-4 already names, re-verified live rather than cited stale. **(b)** No `<bind-src>`/effect verb can target `context.persistent_data` at all — `<bind-src>` closes at exactly `:field`/`:const`/`:metric`/`:tick` and "no §2.8 verb writes anything but a node, an edge, a hyperedge or an event" (`bsl-language.rst:2657-2660`, read directly) — this is a STRICTLY WORSE gap than Metabolism's D-4 (an emit-shaped blocker BSL's vocabulary at least NAMES the target kind for); here no verb even in principle reaches a Python-side dict namespace. **(c)** A declared resolution exists for graph-scope state IN GENERAL — R9 chapter C3's ruling, "graph-scope state is ordinary node state on a declared carrier node type" (`bsl-language.rst:2650-2669`, read directly) — but it is shaped for a SINGLE `:ceiling 1` scalar per carrier node (e.g. one national wealth-pool value), not a multi-key `dict[str, float]` with one entry per commuting-zone/MSA/state. Applying it here would require minting NEW first-class `NodeType`s (`STATE`/`COMMUTING_ZONE`/`MSA`) plus an aggregation `EdgeType` linking each `Territory` to its parents — a primitive-addition, amendment-territory decision (Constitution: "Invent primitives without a constitutional amendment" is a MUST-NOT), outside any port's unilateral scope. **Also a dead channel today** (§5) — zero production consumers exist, so a port may legitimately defer this whole computation with no loss of currently-exercised behavior. |

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_substrate.py` | 523 | **Primary conformance oracle.** `TestCanonicalNoOp` (eligibility no-ops), `TestDepletionMath` (hand-computed, formula-cross-checked: default coefficients, zero extraction, monotone-nonincrease under `regeneration_rate=0`, nonzero-regeneration wiring, ceiling-never-exceeded, `depletion_scale` wiring, zero-floor clamp), `TestCeilingIsARequiredPersistedField` (loud-failure contract on missing/`None` ceiling), `TestOneTickLag` (cross-tick Production→Substrate handoff), `TestScaleLatticeAggregates` (state/nation totality, CZ exclusion + recording, MSA partial-by-design, cross-run determinism, lattice-cached-not-rebuilt), `TestSubstrateSystemIdentity`. Every scenario here is a candidate conformance vector for a hand-built `.bscn` fixture. |
| `tests/unit/engine/laws/test_law_substrate.py` | 287 | **Property-based (Hypothesis) behavioral-contract laws** — genuinely durable "rewrite test" material per this project's testing philosophy. L1 (clamp: post-step stock always in `[0, raw_material_capacity]`, ANY coefficients/inputs), L2 (monotone non-increase under the shipped-default `regeneration_rate=0.0`), L3 (nation-rung extensive-conservation: the published `SUBSTRATE_NATION_KEY` total equals the sum of post-step stocks), L4 (inactivity: no `county_fips` / `None` stock / absent-attribute all write and publish nothing). Explicit non-law caveat recorded in the file's own header: the per-territory clamp (L1) does NOT imply system-wide conservation (mass genuinely enters/leaves via regeneration/ecological cost) — only the default-coefficient case gives the weaker L2. |
| `tests/unit/config/test_substrate_defines.py` | 45 | `SubstrateDefines` Pydantic bounds/defaults contract — schema-level, not tick-behavior. |
| `tests/unit/engine/test_substrate_system_ordering.py` | 44 | Pipeline-ordering: Substrate runs after Territory, before Production. |
| `tests/unit/engine/test_pipeline_substrate_position.py` | 76 | `_DEFAULT_SYSTEMS` registration + slot-position check; documents that the pre-#39-T6 hex pass-through's own dead test (`TestSubstrateZeroPropagation`) was retired because `ProductionSystem` never read `raw_material_stock` (zero grep hits) even then. |
| `tests/integration/test_substrate_pipeline_position.py` | 154 | Live-Postgres integration smoke test. **Its own docstring documents the dormancy finding directly**: every live-pool fixture it exercises today takes the NO-OP path (zero eligible territories) — a live-system confirmation of §5's grep-derived conclusion, not merely a unit-test claim. |

**Excluded as namespace collisions (verified NOT SubstrateSystem's estate by
reading each file's imports — see §1):**

- `tests/integration/economics/test_substrate_pipeline.py` (361 lines) —
  `babylon.domain.economics.substrate.*`, the unrelated tri-county H3-mesh module.
- `tests/unit/economics/substrate/` (14 files + conftest, 4199 lines total) —
  same unrelated module's unit-test estate.
- `tests/unit/persistence/test_substrate_apportionment.py` (176 lines) —
  `babylon.persistence.hex_hydrator`, the different per-hex `dynamic_hex_state`
  persistence-layer `raw_material_stock` column.

**`qa:regression` byte-gate coverage.** `tools/regression_test.py::graph_content_hash`
hashes every node/edge attribute of the `WorldState→graph` projection across
all 12 `SCENARIOS` — but since `SubstrateSystem` no-ops on every one of them
(§5), that gate provides **zero** live coverage of this system's actual
math. All real behavioral coverage today lives in the dedicated
unit/law/integration suite above; a port's conformance fixtures must be
hand-built from scratch (matching Metabolism's own precedent — no canonical
or ad hoc scenario in the current estate seeds `raw_material_stock` at all).

---

## Adjudication (2026-08-12)

Adjudicated against the **current dev tree** (`9324482f`) by a second, read-only
pass, in the manner of the Territory inventory's own "Adjudicated verdict"
section. Three corrections, five confirmations, one coverage note.

1. **CORRECTION — the eligibility-guard workaround cannot be a `bool` field;
   `deffield` has no bool STORAGE type.** §6 row 1 proposes "a seed-time boolean
   discriminator field (e.g. `territory/substrate-eligible`) folding both
   frozen-code conditions into one `bool`", and §3 lists the vocabulary as
   `{int, bool, currency, probability, intensity, coefficient, enum}`. The
   scenario-side declarer admits no `bool`:
   `rust/crates/babylon-bsl/src/scenario.rs:919-931` (`load_deffield`) matches
   exactly `int / probability / intensity / coefficient / currency` plus the
   `enum` branch at `:890`, and its `other =>` arm names that closed list in the
   error text. `bool` reaches only `declarations.rs:649` (`parse_type_name`),
   the typechecker-side name that no `.bscn` field declaration routes through —
   confirmed by `scenario.rs:1071-1078`'s own comment ("`load_deffield` … admits
   only int/probability/intensity/coefficient/currency/enum"). Every landed pack
   records the consequence verbatim:
   `rust/crates/babylon-tick/content/scenarios/vitality-conformance.bscn:19-22`
   — *"0/1 rather than #t/#f: BSL has Bool (§3.1) but `deffield` has no bool
   type and `GraphSubstrate` attributes are f64. Recorded, not hidden."* The
   discriminator must be `(deffield territory/substrate-eligible int extensive)`
   carrying 0/1, an ADR183 declared-deviation row. (The ProductionSystem
   inventory in this same batch caught this independently and correctly; this
   report inherited the task brief's own erroneous vocabulary line.)
2. **CORRECTION — §6 row 1's `BLOCKED` label contradicts this report's own
   executive verdict, and the evidence supports the weaker label.** The blocker
   table says **BLOCKED — no string-identity field type / no field-presence
   test**; the executive verdict two pages earlier says "the eligibility guard
   needs a NEW D-record". Both cannot stand. Adjudicated to **PORTABLE WITH
   D-RECORD**: the guard's two null tests are indeed inexpressible — `Str` is
   typechecker-only and confined to `:material-basis`/vector ids
   (`docs/reference/bsl-language.rst:2394-2398`, read directly), and
   `:optional` requires `:default` so that *"there is consequently no `bound?`
   predicate in the language"* (`bsl-language.rst:2628-2633`, read directly) —
   but a BSL node can never *hold* a null, so the frozen guard's discriminating
   behaviour on any seedable scenario is exactly reproduced by a seed-time
   `int` 0/1 discriminator. What is genuinely unrepresentable (present-but-null)
   is also unreachable. The D-record must record both halves: the encoding, and
   the fact that the null case is dropped because BSL cannot construct it.
3. **CORRECTION — §5's mechanism sentence for the electoral goldens is wrong;
   its count is right.** §5 states the five electoral goldens are "all built on
   `create_two_node_scenario()` + a `model_copy` adding `county_fips`". Only two
   are: `weimar` (`src/babylon/engine/scenarios/electoral_goldens.py:331-332`)
   and `debs` (`:423-424`). `mitterrand` (`:225`), `syriza` (`:270`, same
   idiom) and `bernie_valve` (`:498`) build on
   `create_single_county_scenario()` and inherit `county_fips` from
   `src/babylon/engine/scenarios/single_county.py:116`. The conclusion — six of
   the twelve carry `county_fips="26163"` — stands as written and is confirmed
   below.
4. **CONFIRMATION — the 12-scenario count and the total-no-op finding, both
   re-derived independently.** `tools/regression_scenarios.py:37-133` declares
   exactly 12 `SCENARIOS` keys (`imperial_circuit, two_node, starvation, glut,
   fascist_bifurcation, single_county, mitterrand, syriza, weimar, debs,
   bernie_valve, org_probe`), and `create_scenario`'s factory dispatch
   (`:167-190`) names nine factories, `create_us_scenario` among none of them.
   `raw_material_stock`/`raw_material_capacity` are set in exactly one
   production site repo-wide — `_create_us_territories`,
   `src/babylon/engine/scenarios/_legacy.py:825-826`. The report's conclusion is
   correct and is in fact **stronger than stated**: the early return at
   `src/babylon/engine/systems/substrate.py:218-219` (`if not eligible:
   return`) precedes the lattice build, so Steps 3 and 4 are *unreachable* on
   the canonical estate, not merely inert.
5. **CONFIRMATION — `(domain :graph)` is unserved at execution, verified live
   rather than inherited.** `rust/crates/babylon-bsl/src/tick.rs:524-556`
   (`run_tick`) derives its subject type solely from
   `subject_type_of(&loaded.bindings)` (`:536`); `rg -n "\.domain"
   rust/crates/babylon-bsl/src/tick.rs` returns **zero** hits. Metabolism's D-4
   class transfers exactly as the report claims.
6. **CONFIRMATION, with added teeth — the R9 chapter-C3 escape hatch §6 names
   is itself unserved, so the aggregate-publish blocker is harder than
   described.** The report correctly says C3's carrier-node ruling is shaped for
   a single `:ceiling 1` scalar. It is also **not evaluable today at all**: C3
   prescribes reading with `(field-of (the NodeType/…) …)` and writing with
   `(update-node (the NodeType/…) …)` (`bsl-language.rst:2664-2668`), and `the`
   sits in `UNSERVED_EXPRESSION_HEADS` under **slice 2**
   (`rust/crates/babylon-bsl/src/evaluator.rs:504-506`). Independently
   confirmed: there is no graph-level attribute construct anywhere in either
   Rust crate (`rg -n "graph_attr|graph_attribute|GraphAttr"
   rust/crates/babylon-graph/src/ rust/crates/babylon-bsl/src/` → 0 hits). The
   NOT-A-PACK / BLOCKED split for the lattice + aggregate half stands,
   reinforced.
7. **CONFIRMATION — the cross-system channel table's two load-bearing claims
   spot-check clean.** (a) `rg -n "raw_material_stock"
   src/babylon/engine/systems/` returns `substrate.py` only — no downstream
   System reads it. (b) `rg -n "substrate\.cz|substrate\.msa|substrate\.state|
   substrate\.nation|SUBSTRATE_CZ_KEY|SUBSTRATE_NATION_KEY" src/babylon/`
   returns `substrate.py` only. Both dead-channel findings hold. Also confirmed:
   `extraction_intensity` is read by `substrate.py:229` and written by
   `production.py:246-268`, with `metabolism.py:98,109` the third party — the
   parallel-stock reading in §5 is correct.
8. **CONFIRMATION — tick position 2.5.** `src/babylon/engine/systems/substrate.py:162`
   (`position: ClassVar[float] = 2.5`), against the 34-member `_SYSTEM_CLASSES`
   at `src/babylon/engine/simulation_engine.py:328-363` and the neighbours
   `territory.py:59` (2.0) / `production.py:68` (3.0). Note the registry is
   MEMBERSHIP-only — the order is derived by sorting on `position`
   (`simulation_engine.py:376-377`), so the position ClassVar, not tuple index,
   is the authority. Verdict unchanged.
9. **CONFIRMATION — this system dodges D102, one of only two in this batch.**
   `field-of` on an `:enum-type`-declared field is REFUSED AT LOAD
   (`rust/crates/babylon-bsl/src/typecheck.rs:266-289`,
   `check_no_field_of_on_enum_field`). §3's "zero enum discriminants, zero
   bools" finding means no fold body in a Substrate pack ever needs an
   enum-valued `field-of` — a genuinely favourable structural fact the report
   identified without yet knowing it was load-bearing.

**FINAL VERDICT: SPLIT DISPOSITION — CONFIRMED, with the eligibility guard
upgraded and the aggregate half hardened.** The per-territory depletion rule is
PORTABLE WITH D-RECORDS (Metabolism's D-1 precedent, re-verified live); the
eligibility guard is **PORTABLE WITH D-RECORD** (a seed-time `int` 0/1
discriminator — *not* `bool`, correction 1 — with the null semantics recorded as
dropped-because-unconstructible, correction 2), not BLOCKED; the scale-lattice
build stays NOT-A-PACK (static lookup estate) and the aggregate publish stays
BLOCKED on graph-scope multi-key storage, now doubly so because C3's own
accessor `the` is slice-2 unserved. Dormancy total on all 12 canonical
scenarios, re-derived independently.

**COVERAGE NOTE (not inadequacy).** The inventory files no RESERVED-LINE finding
at all (`grep -c -i "RESERVED-LINE"` → 0), where the ProductionSystem inventory
in this same batch files an explicit negative. On this adjudicator's read the
correct answer is genuinely NONE — `depletion_scale`/`regeneration_rate`/
`entropy_factor` are ecological engineering coefficients, the system carries no
doctrine content, no National Question parameter and no outcome definition — but
a Phase-1 inventory owes the explicit statement, not silence. A re-read adds one
short RESERVED-LINE section recording NONE and the reasoning; nothing else in
the report needs reopening.
