# ContradictionFieldSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `ContradictionFieldSystem` (`src/babylon/engine/systems/contradiction_field.py`,
259 lines, System #19, tick position 19.0) has **two entirely different computations behind one
`step()`**, gated on whether `services.field_registry` is wired. Production **never** wires it
(default `None`, `engine/services.py:196`; grep-confirmed zero non-test call sites pass
`field_registry=`) — so the sole live behavior is the `else` branch, `_step_from_oppositions` (the
"E0 repoint"), which writes exactly two fields per `social_class` node: `"exploitation"` (the MEAN
of this-tick-fresh `tension` over incident `EXPLOITATION`/`WAGES`/`TENANCY` edges, written one
position earlier by `ContradictionSystem@18`) and `"atomization"` (a single graph-level opposition
gap, also from `@18`). **Both of the live path's two field computations are BLOCKED on the current
BSL surface** — `exploitation` needs a typed edge-attribute read (Slice 2's `edges`/`edge-between`
lane; the landed `fold`-over-`neighbors` path cannot substitute, by the language's own ruling that
`neighbors` folds per-node, never per-edge), and `atomization` needs a graph-scope scalar read for
which BSL's only named mechanism (`:metric`) is explicitly refused at load ("slice 1 registers no
metric provider"), while the R9 §3.6-ruled workaround (a singleton carrier `NodeType` + `(the …)`)
needs `the`, itself unserved and bucketed in the very same Slice 2 gap. The two dict-shaped outputs
(`contradiction_fields` per node, `contradiction_history`'s 3-tick rolling window in
`persistent_data`) have no BSL storage shape either way — the closed `deffield` vocabulary is seven
scalar/enum kinds with no map or list type — so a content-modeling D-record is needed regardless of
whether the two hard blockers above ever clear. The dormant `field_registry` path (never reached in
production; exercised only by the primary unit-test class and the integration test) additionally
carries a `math.exp` libm hazard in one of its four default normalizers; the live path is entirely
free of transcendentals. Zero `EventType` emissions on either path.

**Verdict: BLOCKED** — the system's entire live production output sits on two independent unbuilt
lanes (typed edge-attribute reads; a graph-scope-metric/singleton-carrier read), both currently
gated behind Slice 2 of the query-evaluation plan.

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/contradiction_field.py` | 259 | **The target.** `ContradictionFieldSystem`. One `step()` (70-89) branching on `services.field_registry`: the registry path (91-152, DORMANT in production) and `_step_from_oppositions` (153-202, LIVE) plus its two static helpers `_build_tension_index` (204-243) and `_atomization_gap` (245-259). Read completely, line by line. |
| `src/babylon/engine/field_registry.py` | 298 | `FieldRegistryProtocol`, `DefaultFieldRegistry`, and the four default field computations (`compute_exploitation` 88-108, `compute_immiseration` 111-128, `compute_imperial_rent` 131-144, `compute_displacement` 147-165) + two normalizers (`_normalize_linear_10` 173-179, `_normalize_imperial_rent` 182-194, the **libm hazard** — `math.exp` at line 192). Imported/called only by the registry branch of `step()`, which no production code path reaches. |
| `src/babylon/engine/systems/contradiction.py` | 1127 | `ContradictionSystem`, System #18, ONE tick position earlier. `_write_edge_tensions` (192-215) writes fresh `tension` on every `EXPLOITATION`/`WAGES`/`TENANCY` edge (`_TENSION_EDGE_TYPES`, 147-151 — identical tuple/order to this system's own `_FIELD_EDGE_TYPES`). `_step_registry` (221-264) writes the graph-level `opposition_states` attr (256-258, `graph.set_graph_attr`) that `_atomization_gap` reads. This is the **entire same-tick input surface** of the live path — every read `_step_from_oppositions` performs traces to this one file. |
| `src/babylon/formulas/contradiction.py` | 154 | `calculate_wealth_asymmetry_gap` (20- ) — the pure function behind `tension`; scale-free `\|W_b-W_a\|/(W_a+W_b)`, clamped `[0,1]` by construction (docstring, confirmed no `exp`/`log`). |
| `src/babylon/domain/dialectics/core/opposition.py` | 620 | `GapReading` (92-103) — `gap: Intensity` (100), the declared type/domain for the value `_atomization_gap` reads off `opposition_states["atomization"]["gap"]`. |
| `src/babylon/domain/dialectics/instances/catalog.py` | 1272 | `build_default_registry` (808-) — the production opposition registry (eighteen bindings, auto-built whenever `opposition_registry` is `None`, confirming `atomization` is live, not conditionally wired); `_atomization_measure` (427-437) computes the gap this system's live path ultimately reads (out of scope for this port — belongs to `ContradictionSystem`'s own dossier). |
| `src/babylon/engine/systems/field_derivative.py` | 457 | `FieldDerivativeSystem`, System #20, ONE tick position later. **The only downstream reader** of `contradiction_fields`/`contradiction_history` within `src/babylon/engine/systems/` (grep-confirmed). |
| `src/babylon/config/defines/consciousness.py` | 582 (whole module); `ContradictionFieldDefines` 272-331 | Coefficient source. Only `field_min` (285-289) and `field_max` (290-294) are read by this system; `history_window` (297-302), `curvature_alpha` (305-310) and `default_transition_priority` (327-331) are declared but **read nowhere in `src/babylon`** (grep-confirmed) — dead defines, not consumed by this system or any other; `co_optive_suppression_rate` (313-318) and `latent_release_multiplier` (319-324) are read only by `edge_transition/_legacy.py`, not this system. |
| `src/babylon/data/defines.yaml` | `contradiction_field:` block, lines 401-408 | Player-editable coefficient values. |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase._get_persistent_data` (199-202, both branches call it). `_write_clamped` (162-192) exists but **is never called** by this system — both branches reimplement the identical `max(lo, min(hi, v))` clamp inline (126, 193) instead of reusing the shared helper — see §4. |
| `src/babylon/kernel/graph_protocol.py` | 494 | `GraphProtocol.query_nodes` (258-276), `.query_edges` (278-...), `.update_node` (88-98), `.get_graph_attr` (350-362) — every graph verb this system calls. Never calls `get_node`/`update_edge`/`set_graph_attr` (those are `ContradictionSystem`'s). |
| `src/babylon/kernel/tick_partition.py` | 30 | `TickPartition.CONSEQUENCE` — contradiction_field.py:62. |
| `src/babylon/kernel/services.py` | 88 | `ServicesProtocol.field_registry: Any` (43) — structurally `Any`, defaults to `None` at the concrete container. |
| `src/babylon/topology/graph.py` | 1033 | Concrete `BabylonGraph.update_node` (660-668, plain dict merge, no coercion/quantization — same runtime-storage caveat as every other ported system) and `.get_graph_attr`/`.set_graph_attr` (892-898, a plain `dict[str, Any]` keyed store with NO relation to node/edge storage at all). |
| `src/babylon/topology/adapters/query_mixin.py` | 146 | `QueryMixin.query_nodes` (34-), `.query_edges` (70-) — the concrete iteration `BabylonGraph` inherits. |
| `src/babylon/models/enums/topology.py` | 253 | `NodeType.SOCIAL_CLASS = "social_class"` (62); `EdgeType.EXPLOITATION` (99), `.WAGES` (104), `.TENANCY` (106) — the three field-edge types. |
| `src/babylon/models/entities/relationship.py` | 186 | `Relationship.tension: Intensity` (98-101, domain `[0,1]`) — the declared type/domain for the edge attribute this system reads (written by `ContradictionSystem`, not by this system). |
| `src/babylon/models/entities/social_class.py` | 522 | `SocialClass` model, `model_config = ConfigDict(extra="forbid", frozen=True, ...)` (201-205). Declares `wealth: Currency` (307-310), `s_bio: Currency` (386-390), `s_class: Currency` (391-395), `unearned_increment: Currency` (369-372), `population: int` (406-409) — all read ONLY by the dormant registry path. **Does NOT declare `contradiction_fields`** — confirmed absent; see §3. |
| `src/babylon/models/types.py` | 337 | `Currency` (104-, `Annotated[float, ge=0.0]`), `Intensity` (130-, `Annotated[float, ge=0.0, le=1.0]`), `Probability` (50-) — the constrained-float taxonomy behind every domain cited above. |
| `src/babylon/models/world_state.py` | 1161 | `field_stack` (555-) round-trip carrier + `_restamp_field_stack` (831-) — the mechanism that lets `contradiction_fields`/`field_derivatives`/`field_gradients` survive `WorldState.to_graph()`/`from_graph()` despite not being declared `SocialClass`/`Relationship` fields (§3, §5). |
| `src/babylon/sentinels/vocabulary/registry.py` | 754 | `EXTRA_STAMPABLE_ATTRIBUTES[NodeType.SOCIAL_CLASS.value]` (201-209) cites `"contradiction_fields"  # engine/systems/contradiction_field.py` (204) — the formal, sentinel-governed confirmation that this is a graph-only, non-Pydantic attribute. |
| `src/babylon/sentinels/seam/registry.py` | 3201 | FIELD_STATE scope comment (1579-1616) + the `contradiction_fields` SeamEntry (1663-1693): `liveness_class=MUST_BE_LIVE`, write site named explicitly as `_step_from_oppositions (:191, graph.update_node)`, and the notes confirm production sources are exactly `"exploitation" (mean fresh EXPLOITATION/WAGES/TENANCY edge tension)` and `"atomization" (global opposition gap)` "from the E0 opposition-layer repoint, not a field_registry (dormant in production)" — independent, pre-existing confirmation of this report's own read. |
| `src/babylon/engine/services.py` | 459 | `field_registry: Any = field(default=None)` (196, never auto-built, unlike `opposition_registry` which IS auto-built at 382-385 via `build_default_registry(...)` whenever `None` is passed — the asymmetry that makes the live path unconditional). |
| `src/babylon/engine/simulation_engine.py` | 611 | `_SYSTEM_CLASSES` (328-363) confirms tick position: `MarketScissorsSystem@17.8 → ContradictionSystem@18.0 → ContradictionFieldSystem@19.0 → FieldDerivativeSystem@20.0 → CollapseTransitionSystem@20.5 → ...`. |

**Not exercised by contradiction_field.py at all:** no `src/babylon/domain/*` module is imported
directly (the opposition-layer math it depends on lives one system upstream, in
`ContradictionSystem`'s own call graph). No `src/babylon/formulas/*` module is imported directly
either — `calculate_wealth_asymmetry_gap` is `ContradictionSystem`'s import, not this system's.

**Reference BSL/plan sources read for the blocker adjudication** (all fully read/grepped, dev tree):
- `rust/crates/babylon-bsl/src/evaluator.rs` — `SERVED_QUERY_HEADS` (527: `["nodes", "neighbors"]`),
  `UNSERVED_EXPRESSION_HEADS` (503-512: `edges`/`edge-between`/`the` → "slice 2"; `hyperedges`/
  `members-of`/`hyperedges-of`/`metric-of` → "slice 3"; `membership-field-of` → "slice 4").
- `rust/crates/babylon-bsl/src/tick.rs` — `check_sources_servable` (426-466): `BindSource::Metric`
  is refused at **load**, not merely evaluation (438-441, exact message quoted in §6).
- `docs/reference/bsl-language.rst` §3.1 (2293-2385, the closed 7-type `deffield` vocabulary), §3.6
  (2639-2681, the R9 chapter C3 "graph-scope state is ordinary node state on a declared carrier node
  type" draft ruling), and the R9 chapter C8 `neighbors` ruling (1091-1097: "a fold over `neighbors`
  counts and sums per node, never per edge... folds over `edges` instead").
- `rust/crates/babylon-bsl/src/declarations.rs:110` — `DECLARABLE_INTRINSICS: ["exp", "log", "floor"]`.

## 2. COMPUTATION CATALOG (execution order, contradiction_field.py:70-89)

### Computation 1 — Registry-sourced fields (DORMANT — `field_registry` never wired in production)

- **(a)** When a `field_registry` service IS supplied (test-only construction), iterate every
  `social_class` node, compute each registered field's raw value from the node's own attributes
  (plus two synthetic previous-tick values injected from `persistent_data`), normalize each to
  `[field_min, field_max]`, clamp, log a warning on any clamp that actually fired, write the whole
  dict to `contradiction_fields`, and append each field's clamped value to a 3-tick rolling history.
- **(b)** Per default-registered field (`DefaultFieldRegistry.with_defaults()`, field_registry.py:293-298):
  - `exploitation` — `compute_exploitation` (field_registry.py:88-108): `wealth <= 0 → 5.0`, else
    `max(0.0, (subsistence - wealth) / max(subsistence, 0.01))` where
    `subsistence = s_bio + s_class` (default `s_bio=0.01`). Normalized via `_normalize_linear_10`
    (173-179): `max(0.0, min(10.0, raw * 10.0))`.
  - `immiseration` — `compute_immiseration` (111-128): `prev_wealth <= 0 → 0.0`, else
    `max(0.0, prev_wealth - wealth) / prev_wealth`. Same linear-×10 normalizer. Needs
    `_previous_wealth`, injected at contradiction_field.py:115 from `persistent_data`
    (`"_field_previous_wealth"`, keyed by node id, stashed at the END of every prior run, line 150).
  - `imperial_rent` — `compute_imperial_rent` (131-144): `max(0.0, unearned_increment)`. Normalized
    via `_normalize_imperial_rent` (182-194): `raw <= 0 → 0.0`, else
    `max(0.0, min(10.0, 10.0 * (1.0 - math.exp(-raw / 10.0))))` — **the libm hazard** (`import math`
    at line 192, local import, `math.exp` at line 194).
  - `displacement` — `compute_displacement` (147-165): `prev_pop <= 0 → 0.0`, else
    `max(0.0, (prev_pop - pop) / prev_pop)`. Same linear normalizer. Needs `_previous_population`
    (contradiction_field.py:116-118, `persistent_data["_field_previous_population"]`).
- **(c) Reads:** `SOCIAL_CLASS.wealth`, `.s_bio`, `.s_class`, `.unearned_increment`, `.population`
  (all `attrs.get(..., default)`, contradiction_field.py:112,115-118); `persistent_data["_field_previous_wealth"]`/`["_field_previous_population"]` (stashed by this system's own prior tick, not read from any other system).
- **(d) Writes:** `SOCIAL_CLASS.contradiction_fields` (dict, contradiction_field.py:138);
  `persistent_data["contradiction_history"]` (141-147); `persistent_data["_field_previous_wealth"]`/`["_field_previous_population"]` (150-151).
- **(e) Defines:** `contradiction_field.field_min` (0.0, `≥0.0`), `.field_max` (10.0, `>0.0`) —
  defines.yaml:402-403, `ContradictionFieldDefines` consciousness.py:285-294.
- **(f) Events:** none.
- **Status: DORMANT.** `services.field_registry` defaults to `None` (`engine/services.py:196`) and
  is never auto-built (contrast `opposition_registry`, engine/services.py:382-385) nor set by any
  non-test call site (grep-confirmed: every `field_registry=` assignment in the tree is in
  `tests/`). This entire computation never executes in a real simulation run.

### Computation 2 — `exploitation` field (LIVE — production default, `_step_from_oppositions`)

- **(a)** For every `social_class` node, `exploitation` is the arithmetic MEAN of the fresh
  `tension` values on its incident `EXPLOITATION`/`WAGES`/`TENANCY` edges (written by
  `ContradictionSystem@18` the same tick, immediately prior); a node with no incident field edge
  gets `0.0`.
- **(b)** `exploitation = sum(tensions) / len(tensions) if tensions else 0.0`
  (contradiction_field.py:187), where `tensions` comes from `_build_tension_index` (204-243): a
  single O(N+M) pass — outer loop over `_FIELD_EDGE_TYPES` (40-44, fixed tuple order
  `EXPLOITATION, WAGES, TENANCY`, identical order to `ContradictionSystem`'s own `_TENSION_EDGE_TYPES`,
  contradiction.py:147-151), inner loop over `graph.query_edges(edge_type=edge_type)` (233), reading
  `edge.attributes.get("tension")` (234, `isinstance` guard, skip non-numeric), appending to BOTH
  endpoints' lists (240-242, self-loop counted once). Then clamped:
  `max(field_min, min(field_max, exploitation))` (contradiction_field.py:192-194).
- **(c) Reads:** `EdgeType.EXPLOITATION`/`.WAGES`/`.TENANCY` edges' `tension` attribute
  (`Intensity`, `[0,1]`, `models/entities/relationship.py:98-101`) — written by `ContradictionSystem`
  at `contradiction.py:215` the same tick, one position earlier. `NodeType.SOCIAL_CLASS` node
  existence only (`node.id`, not `node.attributes` — the loop at 184-195 never reads a node
  attribute for this computation).
- **(d) Writes:** `SOCIAL_CLASS.contradiction_fields["exploitation"]` (part of the dict written at
  contradiction_field.py:195).
- **(e) Defines:** `contradiction_field.field_min`/`.field_max` (same as Computation 1's e).
- **(f) Events:** none.
- **Status: LIVE**, and (§5) exercised on every canonical `qa:regression` scenario — none of them
  are dormant on this input, unlike Territory's ADJACENCY gap.

### Computation 3 — `atomization` field (LIVE — production default, `_step_from_oppositions`)

- **(a)** `atomization` is a single graph-wide value (the atomization opposition's current gap,
  `[0,1]`) applied uniformly to every `social_class` node this phase — no per-node variation.
- **(b)** `_atomization_gap` (245-259): `states = graph.get_graph_attr("opposition_states", {}) or {}`
  (256); `atomization = states.get("atomization", {})` (257); `raw = atomization.get("gap", 0.0)`
  (258); `return float(raw) if isinstance(raw, (int, float)) else 0.0` (259). Then clamped identically
  to Computation 2 (contradiction_field.py:192-194, same dict comprehension, shared with
  `exploitation`).
- **(c) Reads:** the graph-level attribute `opposition_states` (a plain `dict[str, Any]`, keyed by
  opposition name, written by `ContradictionSystem._step_registry` at `contradiction.py:256-258` via
  `graph.set_graph_attr`, one tick position earlier) — specifically its `"atomization"` entry's
  `"gap"` key. The upstream `GapReading.gap: Intensity` type (`domain/dialectics/core/opposition.py:100`)
  gives the declared domain `[0,1]`. `atomization` is one of eighteen bindings the production
  `opposition_registry` always includes (`build_default_registry`, catalog.py:808-, auto-built
  whenever `None` is passed — engine/services.py:382-385), so this key is present on every real run
  once `ContradictionSystem` has run at least once this tick.
- **(d) Writes:** `SOCIAL_CLASS.contradiction_fields["atomization"]` (same dict write as
  Computation 2, contradiction_field.py:195) — identical value on every node this tick (uniform
  broadcast, not per-node computation).
- **(e) Defines:** same `field_min`/`field_max` as above.
- **(f) Events:** none.
- **Status: LIVE.**

### Computation 4 — 3-tick rolling history write (both branches, shared shape)

- **(a)** After computing each field's clamped value, append it to a per-node, per-field list in
  `persistent_data["contradiction_history"]`, trimmed to the last `_MAX_HISTORY_WINDOW` (module
  constant, `= 3`, contradiction_field.py:36) entries — a plain shift-window, oldest dropped first.
- **(b)** `field_history.append(value); while len(field_history) > _MAX_HISTORY_WINDOW: field_history.pop(0)`
  (registry path: 141-147; opposition path: 197-202, iterating the fixed `_OPPOSITION_FIELD_NAMES`
  tuple, 47: `("exploitation", "atomization")`, deterministic order).
- **(c) Reads:** the SAME `persistent_data["contradiction_history"]` dict from the prior tick
  (accumulator, not a fresh read of engine state).
- **(d) Writes:** `persistent_data["contradiction_history"][node_id][field_name]` (a `list[float]`,
  length ≤ 3). **Not a graph attribute at all** — lives entirely in `TickContext.persistent_data`,
  a plain Python dict the engine host carries across ticks, orthogonal to node/edge/graph-attr
  storage.
- **(e) Defines:** `_MAX_HISTORY_WINDOW = 3` is a **module-level Python constant, not a define** —
  note the `contradiction_field.history_window` define exists in `GameDefines`
  (default `3`, domain `[2,10]`, consciousness.py:297-302) but is **never read** by this constant
  or anywhere else in `src/babylon` (grep-confirmed) — a live/dead-value mismatch: changing the
  `history_window` define in `defines.yaml` has zero effect on this system's actual window size.
- **(f) Events:** none.
- **Consumer:** `FieldDerivativeSystem@20` reads `contradiction_history` for `df_dt`/`d2f_dt2`
  (field_derivative.py, confirmed by grep — out of scope for this port).

**Events emitted by the whole system: zero.** Confirmed by grep — no `EventType`/`_publish`/
`event_bus`/`.publish(` reference anywhere in contradiction_field.py.

## 3. TYPE INVENTORY

| Attribute | Node/scope | Python model type | Domain | Category |
|---|---|---|---|---|
| `contradiction_fields` | SOCIAL_CLASS | `dict[str, float]` — **not a declared `SocialClass` Pydantic field** (`extra="forbid"`, confirmed absent; sanctioned via `EXTRA_STAMPABLE_ATTRIBUTES`, `sentinels/vocabulary/registry.py:204`) | keys `{"exploitation","atomization"}` (live path) or the four registry-path names; values `[field_min, field_max]` = `[0.0, 10.0]` by define, but the live path's actual values only ever populate `[0,1]` in practice (both inputs are `Intensity`-typed) | **dict-valued attribute — no closed-vocabulary BSL storage type** |
| `contradiction_history` | `persistent_data` (TickContext, host-side, **not graph-scoped at all**) | `dict[str, dict[str, list[float]]]` | per-list length `≤ 3`, values same domain as above | **nested dict-of-lists, non-graph-scoped — no BSL analog of any kind** |
| `tension` | EdgeType.EXPLOITATION / .WAGES / .TENANCY (edge attribute, read only) | `Intensity` (`models/entities/relationship.py:98-101`) | `[0.0, 1.0]` | **EdgeRef attribute — needs Slice 2 (`edges`)** |
| `opposition_states` | graph-level attribute (`graph.get_graph_attr`, read only) | `dict[str, dict[str, Any]]` (values are `OppositionState.model_dump()`, catalog is 18-keyed) | this system reads only `["atomization"]["gap"]`, an `Intensity` `[0,1]` per `GapReading.gap` | **graph-scope scalar — no BSL storage/read mechanism landed (see §6)** |
| `field_min` (define) | — | `float`, `ge=0.0` | `[0.0, ∞)` in principle, `0.0` by default | coefficient |
| `field_max` (define) | — | `float`, `gt=0.0` | `(0.0, ∞)`, **unbounded above**, `10.0` by default | **unbounded-above coefficient** |
| `history_window` (define) | — | `int`, `ge=2, le=10` | `[2,10]`, `3` by default | coefficient — **dead**: never read anywhere (§2, computation 4) |
| `wealth`, `s_bio`, `s_class`, `unearned_increment` (registry path only) | SOCIAL_CLASS | `Currency` (`Annotated[float, ge=0.0]`, `models/entities/social_class.py:307-310,369-372,386-395`) | `[0.0, ∞)` | unbounded real, money/quantity-semantic — dormant-path only |
| `population` (registry path only) | SOCIAL_CLASS | `int`, `ge=0` (`social_class.py:406-409`) | `≥0` | integer — dormant-path only |
| `_previous_wealth`, `_previous_population` (registry path only) | synthetic, injected into a local `attrs` dict copy, never a real node attribute | `float` | same as `wealth`/`population` | **host-computed cross-tick memory, not a graph attribute** — dormant-path only |

**No enum discriminants read or written by this system** — unlike Territory, ContradictionFieldSystem
has no `StrEnum`-typed field on either its read or write surface.

**No bools read or written.**

**The dict-valued-attribute flag is a genuine, previously-unnamed gap for THIS system** (distinct
from Territory's enum-storage gap): `deffield`'s closed seven-type vocabulary
(`int, bool, currency, probability, intensity, coefficient, enum` — `bsl-language.rst` §3.1,
2293-2385) has no map/list/record type, and the document says so explicitly: "There are no type
variables, no subtyping, no coercions, and no user-defined types" (2367-2368). A content-modeling
workaround (same class as Territory's enum→bool/ordinal, Metabolism's bare-scaled-Int) exists in
principle — decompose `contradiction_fields` into two named scalar `deffield`s
(`social-class/field-exploitation`, `social-class/field-atomization`) and `contradiction_history`
into `2 fields × 3 slots = 6` named scalar `deffield`s with shift-effects each tick — but it is a
genuine transcription decision with no existing precedent in a landed pack, requiring its own
D-record, independent of whether the two hard blockers in §6 ever clear.

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `float`). Shapes, in execution order (live path first,
since that is the actual port target; registry path noted separately):

**Live path (`_step_from_oppositions`):**
1. **Mean-of-list reduction:** `sum(tensions) / len(tensions) if tensions else 0.0`
   (contradiction_field.py:187) — one sum, one divide, guarded against empty-list division by the
   ternary. No transcendentals.
2. **Dict-value clamp (×2 keys, one shared expression):**
   `{name: max(field_min, min(field_max, value)) for name, value in raw.items()}`
   (contradiction_field.py:192-194) — `max(min(...))`, structurally identical to
   `SystemBase._write_clamped`'s own `max(lo, min(hi, value))` (system_base.py:189), **but
   reimplemented inline rather than calling the shared helper** — an available-but-unused-helper
   oddity, not a defect (the two shapes are byte-identical, just not code-shared).
3. **Float coercion guards:** `float(raw) if isinstance(raw, (int, float)) else 0.0`
   (contradiction_field.py:237 for tension; :259 for the atomization gap) — no arithmetic, pure
   type-narrowing before the arithmetic above.
4. **Bare non-integer literal:** `0.0` appears as the ternary fallback (187) and the `isinstance`
   guard fallback (237, 259) — under the BSL "no bare non-integer literal" constraint (per the
   Territory/Metabolism precedent packs), each site needs a `c`-suffixed constant or the Real-zero
   promotion idiom already used elsewhere.
5. **No Real→Int demotion anywhere** on the live path (unlike Territory's `int(...)` displacement/
   decay sites) — every live-path value stays `float` end to end.
6. **Clamp bounds are practically dead weight on the live path:** `field_min=0.0`/`field_max=10.0`
   are defined for the registry path's `_normalize_linear_10`/`_normalize_imperial_rent` outputs
   (which DO range up to 10.0), but the live path's two inputs (`tension`, `opposition_states[...]
   .gap`) are both `Intensity`-typed `[0,1]` — so on every real run the clamp's upper bound
   (`10.0`) is structurally unreachable and the lower bound (`0.0`) is unreachable too (mean of
   non-negative values, and `float(raw)` on an already-non-negative gap). Not a bug — the same
   clamp constants correctly serve both branches' different ranges — but worth naming: the live
   path's clamp never actually fires.

**Registry path (`step()`'s `if registry is not None` branch, DORMANT):**
7. **Subsistence-deficit ratio:** `max(0.0, (denominator - wealth) / denominator)`
   (field_registry.py:108) where `denominator = max(subsistence, 0.01)` (107) — a floor-guarded
   divide, no transcendentals.
8. **Decline-rate ratio (×2, immiseration/displacement):** `max(0.0, prev - cur) / prev`
   (field_registry.py:127, 165) — same shape, guarded by `prev_wealth <= 0`/`prev_pop <= 0` early
   returns (125, 162).
9. **Linear scale-and-clamp:** `max(0.0, min(10.0, raw_value * 10.0))` (`_normalize_linear_10`,
   173-179) — one multiply, one clamp. Bare literals `10.0`/`0.0` both need the same BSL-literal
   treatment as item 4.
10. **LIBM TRANSCENDENTAL — `math.exp`:** `_normalize_imperial_rent` (182-194):
    `10.0 * (1.0 - math.exp(-raw_value / 10.0))`, clamped to `[0.0, 10.0]` — a genuine
    cross-implementation-nondeterminism hazard (`exp` is a declarable BSL intrinsic per
    `declarations.rs:110`, so it is EXPRESSIBLE, but per Constitution/CLAUDE.md's own tolerance-policy
    rule, any cross-language `exp` reproduction needs an explicit derivation, not an assumed match).
    **This code path never executes in production** (§2, Computation 1's dormancy) — flagged for
    completeness per the honesty rules, not as a live-path hazard.
11. **Real→Int demotion:** none in the registry path either — `population` is read as `int` but
    immediately coerced to `float` for the ratio (field_registry.py:160-161), never written back as
    `int` by this system (the registry path doesn't write `population` at all, only reads it).

**No `sigmoid`/`pow` anywhere in either path.** The only libm hazard in the whole file map is the
single dormant `math.exp` call above; the live production path — the only path that matters for
this port — has **zero** transcendentals, zero Real→Int demotions, and exactly one clamp shape
(reused twice, not duplicated with a second implementation the way Territory's Phase-1/Phase-3
heat clamps diverged).

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 19.0** (contradiction_field.py:63), confirmed against `_SYSTEM_CLASSES`
  (`simulation_engine.py:328-363`): `MarketScissorsSystem (17.8) → ContradictionSystem (18.0) →
  ContradictionFieldSystem (19.0) → FieldDerivativeSystem (20.0) → CollapseTransitionSystem (20.5) →
  ...`. Cross-checked against `tests/unit/engine/test_system_order.py:90-97,259-266` (300 lines,
  positional-order oracle, not a behavioral test of this system's internals).
- **Reads from the immediately-prior same-tick system, exhaustively:** unlike most Consequence-phase
  systems, ContradictionFieldSystem's live path reads NOTHING from `wealth`/`population`/any other
  node-owned economic state directly — its only two inputs are both `ContradictionSystem@18`'s own
  fresh writes, one tick position earlier, same tick: (1) the `tension` attribute on
  `EXPLOITATION`/`WAGES`/`TENANCY` edges (`contradiction.py:215`), and (2) the graph-level
  `opposition_states` attr (`contradiction.py:256-258`). This is a **maximally narrow, single-hop
  input surface** — a favorable structural fact for the port (only one upstream system's output to
  reconcile), even though both of those specific reads are individually blocked (§6).
- **Writes consumed later this tick / downstream ticks:**
  - `SOCIAL_CLASS.contradiction_fields` — read by exactly **one** downstream engine system:
    `FieldDerivativeSystem@20` (grep-confirmed across `src/babylon/engine/systems/*.py`; also
    consumed outside the engine proper by `web/game/engine_bridge.py`'s FIELD_STATE payload
    builder and `projection/`/`sentinels/seam/registry.py`'s `MUST_BE_LIVE` seam entry — out of
    engine scope for this port).
  - `persistent_data["contradiction_history"]` — read by exactly **one** downstream system,
    `FieldDerivativeSystem@20`, for its `df_dt`/`d2f_dt2` temporal derivatives (grep-confirmed).
  - **No SOCIAL_CLASS field this system writes is read by anything else in `engine/systems/`** —
    both writes terminate in `FieldDerivativeSystem`, a single, tightly-coupled two-system pipeline
    (`ContradictionFieldSystem → FieldDerivativeSystem`), unlike Territory's 13-system-wide
    `population` fan-out.
- **Context/service usage with no BSL equivalent:**
  - `services.field_registry` (contradiction_field.py:86) — the DI seam gating the entire dormant
    Computation 1. Never wired in production (§2). Has no BSL analog because it never needs one:
    the dead branch is out of scope for the port entirely.
  - `context.persistent_data` (`_get_persistent_data`, system_base.py:199-202) — the host-side
    dict this system both reads and writes for `contradiction_history` and, on the dormant path
    only, `_field_previous_wealth`/`_field_previous_population`. `docs/reference/bsl-language.rst`
    §3.6 (2650-2657) names `context.persistent_data` explicitly as "the single most pervasive gap
    in the estate" (R9 gap analysis §2, Q6) shared by 22 of 34 frozen systems, and rules a sanctioned
    workaround (a declared carrier `NodeType` with `:ceiling 1`, read via `(field-of (the …) …)`) —
    see §6 for why that workaround is itself currently unbuildable.
  - `graph.get_graph_attr("opposition_states", {})` (contradiction_field.py:256) — the concrete
    `BabylonGraph` implementation of graph-level attributes (`topology/graph.py:892-898`) is a
    plain `dict[str, Any]` with no relation to node/edge storage at all; BSL's `GraphSubstrate` has
    no equivalent construct (confirmed: zero hits for `graph_attr`/`GraphAttr`/"graph-level" across
    `rust/crates/babylon-bsl/src/*.rs` and `babylon-tick/src/*.rs`).
- **DORMANCY on canonical scenarios: NOT dormant — the live path is exercised on every canonical
  scenario.** Unlike Territory (whose ADJACENCY-dependent phases are structurally dormant on every
  `qa:regression` scenario), `EXPLOITATION`/`WAGES`/`TENANCY` edges ARE seeded by every canonical
  scenario's base substrate:
  - `create_two_node_scenario` (`engine/scenarios/_legacy.py:46-201`) seeds `EXPLOITATION` (113),
    `WAGES` (139), `TENANCY` (159) edges directly.
  - `create_imperial_circuit_scenario` (`_legacy.py:255-958`, the `imperial_circuit`/`starvation`/
    `glut` factory used by most `SCENARIOS` rows in `tools/regression_scenarios.py:40,50,57,65`)
    seeds `EXPLOITATION` (410), `WAGES` (430), `TENANCY` (481, 490).
  - `create_single_county_scenario` (`engine/scenarios/single_county.py:98,107,125`) seeds all three
    (used by the `wayne_county` row, `regression_scenarios.py:74`).
  - The five electoral-golden factories (`mitterrand`/`syriza`/`weimar`/`debs`/`bernie_valve`,
    `regression_scenarios.py:80,91,100,109,118`) stand on the Wayne `single_county` or `two_node`
    substrates (`engine/scenarios/electoral_goldens.py:7-16`), inheriting the same edges.
  - `create_org_probe_scenario` (`engine/scenarios/org_probe.py:106`) seeds at least one `TENANCY`
    edge (`regression_scenarios.py:130`).
  - The production `opposition_registry` is auto-built whenever `None` is passed
    (`engine/services.py:382-385`) and always includes `atomization` among its eighteen bindings
    (`catalog.py:808-`), so the graph-level read is live on every run too, once `ContradictionSystem`
    has executed at least one tick.
  - **Consequence for the port:** conformance fixtures for BOTH live-path computations CAN be
    harvested from (or built alongside) the existing canonical scenario substrates, unlike
    Territory's Phase 2-4 (which needed hand-built `.bscn` fixtures because nothing canonical
    exercises them). The blocker here is purely the missing BSL language surface (§6), not
    fixture availability.

## 6. BLOCKER ASSESSMENT

Adjudicated against the CURRENT BSL surface (dev tree, anchors verified above).

| Computation | Verdict | Detail |
|---|---|---|
| Node iteration + dict-write shell (iterate `social_class`, write a computed value) | **PORTABLE WITH D-RECORD** (contradiction_fields decomposition) | `(nodes NodeType/SOCIAL-CLASS)` is a served query head (Slice 1, `SERVED_QUERY_HEADS`, `evaluator.rs:527`); `for-each` in effect position and `update-node` against a computed `NodeRef` are both landed (per the CURRENT BSL surface brief). The dict-valued write itself needs the §3 decomposition D-record (N named scalar `deffield`s in place of one dict), which is mechanical but undocumented in any landed pack. This shell has nothing useful to write until Computations 2 and 3 below unblock. |
| `exploitation` field (mean of incident field-edge `tension`, contradiction_field.py:187) | **BLOCKED — Slice 2 (`edges`/edge-attribute reads)** | Reading `tension` requires iterating EDGES of a given type and reading their attributes — the `edges`/`edge-between` query heads (`UNSERVED_EXPRESSION_HEADS`, `evaluator.rs:503-512`, both "slice 2"). `fold`/`neighbors` (Slice 1, landed) CANNOT substitute: `bsl-language.rst`'s own R9 chapter C8 ruling states explicitly, "a fold over `neighbors` counts and sums per node, never per edge... `neighbors` answers *which nodes*, not *how many ways*" (1091-1097) — precisely the "once per contributing edge" shape this computation needs, which the ruling names as the case that "folds over `edges` instead." This is a structural, not incidental, mismatch. |
| `atomization` field (graph-level `opposition_states["atomization"]["gap"]`, contradiction_field.py:256-259) | **BLOCKED — no graph-scope-attribute lane** | No `graph_attr`-equivalent construct exists in `babylon-graph`/BSL at all (§5). BSL's one named mechanism for a graph-level scalar, the `:metric` binding source, is **explicitly refused at load**: `tick.rs:438-441` — `":metric {name} — slice 1 registers no metric provider; §2.11 providers are Phase-2 kernel services"`. The `bsl-language.rst` §3.6 "draft ruling" sanctioned workaround (a singleton carrier `NodeType`, `:ceiling 1`, read via `(field-of (the …) …)`, 2660-2668) needs `(the …)`, which is ALSO unserved — `("the", "slice 2")` in the same `UNSERVED_EXPRESSION_HEADS` table as the edge lane above. Two independent named routes to this value, both currently closed. |
| 3-tick rolling-history write (`contradiction_history`, persistent_data) | **PORTABLE WITH D-RECORD** (fixed-slot scalar-field decomposition) | Mechanically expressible as `2 fields × 3 slots = 6` named scalar `deffield`s with shift-effects (`newest := computed; slot1 := slot0; slot0 := newest`) each tick — no array/list type exists (§3), but the WINDOW SIZE is fixed at compile time (`_MAX_HISTORY_WINDOW = 3`, never varies), so a fixed unrolling is exact, not an approximation. Requires its own D-record (no landed-pack precedent for this shape) and is moot until Computations 2-3 unblock. |
| Registry-path fields (`exploitation`/`immiseration`/`imperial_rent`/`displacement`, field_registry.py) | **NOT-A-PACK** | Dead code in production — `services.field_registry` is never wired outside `tests/` (§2, Computation 1). Nothing here needs porting; documented for completeness per the honesty/port-as-is rules, not as a port candidate. If ever activated, `immiseration`/`displacement` would ALSO need a cross-tick `_previous_wealth`/`_previous_population` memory this port inventory did not adjudicate (out of scope, dormant), and `imperial_rent`'s normalizer would carry the `math.exp` PORT-QUESTION class (declarable but needing an explicit cross-language tolerance derivation, not a routine D-record). |
| Registry-path `field_min`/`field_max` clamp (shared shape with the live path, contradiction_field.py:126, 192-194) | **PORTABLE NOW** (as an isolated arithmetic shape) | `max(lo, min(hi, v))` with `lo`/`hi` both `[0,1]`-adjacent (`field_min` `ge=0.0`) — trivial nested-`if`, exact precedent in every landed pack (Territory's own Phase-1 clamp). Moot in isolation since it clamps values Computations 2/3 cannot yet produce. |
| `contradiction_field.history_window` define | **NOT-A-PACK (dead, own-system finding)** | Read nowhere in `src/babylon` (§2, Computation 4) — not a port question at all; the port's fixed 3-slot decomposition above needs no define, since `_MAX_HISTORY_WINDOW` is a hardcoded Python constant this define does not actually govern. |

**Overall system verdict: BLOCKED.** Every computation the LIVE production path performs — both of
the two fields it writes — is individually blocked, on two different-but-adjacent missing lanes that
the query-evaluation plan's own bucketing places in the same next increment (Slice 2). The
node-iteration shell and the history-window write are each independently PORTABLE WITH D-RECORD, but
both are downstream of values neither can yet be given.

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_contradiction_field_system.py` | 432 | **Primary conformance-oracle candidate, and the ONLY test file that exercises the LIVE path.** `TestContradictionFieldSystemBasic`/`TestContradictionFieldHistory` (27-256) test the DORMANT registry path exclusively (`DefaultFieldRegistry.with_defaults()` wired every time) — schema/behavioral documentation for dead code, not a port target. `TestContradictionFieldNoRegistry` (259-326) and `TestContradictionFieldTensionIndex` (329-432) are the real prize: `test_exploitation_field_is_mean_incident_tension_not_max` (278-287, a genuine mean-vs-max mutation-killing assertion), `test_atomization_field_is_global_opposition_gap` (289-297), `test_no_edges_no_snapshot_writes_zero_fields` (316-326, the honest-zero EC case), and the 5-node/4-edge O(N+M) fixture (`_populate_field_graph`, 339-361) with exact expected per-node means (372-378) — these ARE the live path's behavioral contract and the direct source for a hand-built `.bscn` conformance fixture once §6's blockers clear. |
| `tests/integration/test_field_topology_integration.py` | 171 | Exercises the DORMANT registry path exclusively (`DefaultFieldRegistry.with_defaults()` at every `ServiceContainer.create(...)` call, grep-confirmed 4/4 sites) across a 3-system pipeline (`ContradictionFieldSystem → FieldDerivativeSystem → EdgeTransitionSystem`). Schema/pipeline-shape documentation for dead-in-production code, not a conformance oracle for what ships. |
| `tests/unit/engine/systems/test_field_derivative_system.py` | 553 | Tests `FieldDerivativeSystem` (System #20), NOT this system — out of scope, but its `test_no_registry_and_no_fields_is_noop`-class tests (264-, 545-548) independently corroborate the "no field_registry" production default this report relies on. |
| `tests/unit/engine/test_system_order.py` | 300 | Cross-system positional-order oracle (§5) — confirms tick position and neighbor systems; not a behavioral test of this system's internals. |

**qa:regression byte-gate coverage.** `tools/regression_test.py::graph_content_hash` (lines 924-964)
hashes every node/edge attribute of the `WorldState → graph` projection, so any change to
`ContradictionFieldSystem`'s live-path outputs on any canonical scenario is caught by the
byte-identical hash gate. Given §5's finding that the live path is NOT dormant on any canonical
scenario (unlike Territory), this coverage is real and complete for both `exploitation` and
`atomization` — no declared coverage gap analogous to Territory's
`tools/regression_scenarios.py:2678-2683` special-type omission exists for this system. A port's
`.bscn` conformance fixture can be built directly from (or as a minimal excerpt of) any canonical
scenario's substrate rather than requiring an entirely hand-built fixture.

## Adjudication (2026-08-12)

Adjudicated against the **current dev tree** (`9324482f`), read-only, with fresh anchors.
The dormancy finding (§5), the dead-`history_window` finding and the `field_registry`
production-dormancy finding all verify and are the report's best work. The **BLOCKED verdict
survives but narrows from two lanes to one**, and that one lane is not the one named.

1. **CORRECTION — the `atomization` row is wrong on the route it dismisses: the §3.6 carrier
   does NOT need `(the …)`, and this computation is portable today under landed Slice 1.**
   §6's row says the §3.6 workaround *"needs `(the …)`, which is ALSO unserved … Two
   independent named routes to this value, both currently closed."* The `:metric` half is
   correct and verified (`tick.rs:438-441`, refused at load: *":metric {name} — slice 1
   registers no metric provider; §2.11 providers are Phase-2 kernel services"*). The §3.6
   half is not. §3.6's ruled *mechanism* is "an ordinary `deffield` owned by a **carrier node
   type** — a `NodeType` member whose manifest `:ceiling` is 1" (`bsl-language.rst:2664-2668`);
   `(the …)` is the sugar the ruling happens to write the read with, not the storage class
   and not the only accessor. Verified on dev:
   - `tick.rs:159-181`'s `subject_type_of` derives a rule's subject `NodeType` from the
     namespace of its `:field` bindings, and `run_tick` iterates `graph.nodes(&subject_type)`
     (`tick.rs:536-538`) — a rule anchored on a ceiling-1 carrier runs over exactly one
     subject, with no accessor form at all.
   - A cross-type read is `(fold sum (nodes NodeType/OPPOSITION-REGISTER) (field-of it
     opposition-register/atomization-gap))` — `nodes` sits in `SERVED_QUERY_HEADS`
     (`evaluator.rs:527`) precisely as a fold operand, and `field-of` over a `NodeRef` is
     landed (`evaluator.rs:1197-1223`). At ceiling 1 that fold **is** `(the …)`.
   - The carrier `NodeType` is content-declarable today: `(defvocabulary NodeType (…))` is a
     landed `.bscn` form (`scenario.rs:389-395`, `load_defvocabulary` `:811-850`), already in
     use at `content/scenarios/organization-foundation.bscn:41`.
   Computation 3 is therefore **PORTABLE WITH D-RECORDS**, not BLOCKED — gated only on
   `ContradictionSystem`@18 writing its `atomization` gap to a carrier node instead of
   `graph.set_graph_attr`, which is that system's own port decision and is itself unblocked
   by the same ruling.

2. **CORRECTION — the `exploitation` row names the wrong lane: the gap is the SUBSTRATE, not
   query slice 2, and slice 2 alone would not touch it.** The row's reasoning about
   `neighbors`-vs-`edges` is exactly right and the R9 chapter C8 citation
   (`bsl-language.rst:1091-1097`) is apt — a fold over `neighbors` counts per node, never per
   edge, so the "once per contributing edge" mean cannot be reformulated. But the row stops
   at "the `edges`/`edge-between` query heads … both 'slice 2'." Landing slice 2 mints edge
   *references*; there is nothing to read off them. The full `GraphSubstrate` trait surface is
   `rust/crates/babylon-graph/src/substrate.rs:80-248`, and it contains **no edge-attribute
   accessor of any kind**: the one edge reader is `fn edges(&self, edge_type: &str) ->
   Vec<(NodeId, NodeId)>` (`:166`), returning bare id pairs, next to
   `fn add_edge(…, strength: f64, …)` (`:111-116`) — no `edge_attribute`, no reader for the
   strength either. `rust/crates/babylon-bsl/src/structural_verbs.rs:387-398` states the same
   fact from the write side verbatim and prices it: *"GraphSubstrate keys an edge to one f64
   strength and gives a hyperedge no attributes at all. Widening that state widens the
   canonical `state_hash` field set, which is a declared Phase-2/substrate decision
   (Constitution III.7), never a silently-dropped write."* The sibling `ContradictionSystem`
   inventory reached this finding for the identical `tension` attribute one file over; this
   one did not, and the difference matters for sequencing — a query-evaluation train clears
   nothing here.

3. **CORRECTION — the history-window decomposition row calls a real hazard "mechanical".**
   §6 files the 3-tick window as *"Mechanically expressible as `2 fields × 3 slots = 6` named
   scalar `deffield`s with shift-effects (`newest := computed; slot1 := slot0; slot0 :=
   newest`) each tick … a fixed unrolling is exact, not an approximation."* The unrolling is
   exact **only if the whole shift chain lives inside one rule's effect list.** `run_tick` is
   a two-pass collect-then-apply over one rule (`tick.rs:524-560`: Pass 1 collects against an
   immutable reborrow `&*graph`; Pass 2 applies), and per the standing brief's open D-row
   Q14/D116 **two rules at one anchor position do not yet share pre-state**. A shift chain
   split across two rules at the same position would read a half-shifted window and silently
   corrupt `df_dt`/`d2f_dt2` downstream. The sibling `FieldDerivativeSystem` inventory names
   exactly this constraint for the same ring buffer (its Phase-2 row cites §2.8's source-order
   guarantee and says so explicitly); this row should carry it too, since it is the difference
   between an exact unroll and a wrong one.

4. **CONFIRMATION — the `field_registry` production-dormancy finding.** Verified
   independently: `rg -n field_registry` over `src/` and `web/` with `tests/` excluded returns
   only the protocol declaration (`kernel/services.py:43`), the `= field(default=None)`
   default (`engine/services.py:196`), the two consuming branches
   (`contradiction_field.py:86`, `field_derivative.py:70`), one `edge_transition` docstring,
   and three independent corroborating comments (`world_state.py:70`,
   `sentinels/seam/registry.py:1584,1684`). Zero non-test assignment sites. Computation 1 is
   dead in production; NOT-A-PACK is the right disposition.

5. **CONFIRMATION — `history_window` is a dead define, and the live/declared mismatch is
   real.** `rg -n history_window` over `src/` (tests excluded) returns exactly two hits: the
   `defines.yaml:404` value and the `config/defines/consciousness.py:297` declaration. No
   reader anywhere. `_MAX_HISTORY_WINDOW = 3` (`contradiction_field.py:36`) genuinely governs
   the window and is not defines-driven. WS4 adjudication row as filed.

6. **CONFIRMATION — the not-dormant finding, and its consequence.** Tick position 19.0
   confirmed (`contradiction_field.py:63`) against `_DEFAULT_SYSTEMS`' position-sorted
   derivation (`simulation_engine.py:376-378`), between `ContradictionSystem` 18.0
   (`contradiction.py:170`) and `FieldDerivativeSystem` 20.0 (`field_derivative.py:47`). The
   `EXPLOITATION`/`WAGES`/`TENANCY` seeding claim spot-checks clean:
   `single_county.py:95-130` seeds all three on the Wayne substrate the five electoral
   goldens inherit, and `_legacy.py:405-450` seeds EXPLOITATION and WAGES on the
   `imperial_circuit` default. §5's conclusion — that conformance fixtures for the live path
   can be harvested from canonical substrates rather than hand-built, unlike Territory's — is
   correct and is a genuinely favourable finding for the port.

7. **CONFIRMATION — the byte-gate claim in §7.** `tools/regression_test.py:924-964`'s
   `graph_content_hash` digests `graph.nodes(data=True)`/`graph.edges(data=True)`
   (`:958-963`), and `contradiction_fields` is a node attribute — so both live-path outputs
   are genuinely hash-covered, with no declared coverage gap. Correct as stated. (Note it
   covers them *via* the field-stack restamp: `world_state.py:855-867`'s
   `_restamp_field_stack` puts `contradiction_fields`/`field_derivatives` back onto nodes
   during `to_graph()`. The coverage is real; its route is one hop longer than §7 implies.)

**FINAL VERDICT: BLOCKED — on ONE lane, not two, and that lane is the substrate, not a query
slice.** Computation 3 (`atomization`), the node-iteration/dict-write shell, the clamp and
the 3-slot history decomposition are all **PORTABLE WITH D-RECORDS on landed Slice 1**, via a
`:ceiling 1` carrier `NodeType` — no `the`, no `:metric`, no Slice 2 (correction 1), with the
single-rule shift-chain constraint declared (correction 3). The sole surviving blocker is
Computation 2 (`exploitation`), whose per-edge `tension` read needs `GraphSubstrate`
edge-attribute **storage** — a hash-relevant Constitution III.7 substrate decision, deeper
than Slice 2 and unscheduled on any of the four named slices (correction 2). Since
`exploitation` is one of only two fields this system writes, the system as a whole remains
BLOCKED; but the named unblock changes from "the next query-evaluation increment" to "a
substrate widening", and the honest partial — an `atomization`-only pack — is now on the
table as a scope question rather than an impossibility.

**INADEQUATE-COVERAGE — a re-read must add:**
(a) `rust/crates/babylon-graph/src/substrate.rs:80-248` (the trait surface) to the
reference-sources list — the inventory read `evaluator.rs`, `tick.rs`, `declarations.rs` and
`bsl-language.rst` but never the file that decides whether an edge attribute exists at all,
which is what its central blocker turns on;
(b) `tick.rs`'s `subject_type_of`/`run_tick` (`:159-181`, `:524-560`) and `scenario.rs`'s
`defvocabulary` path (`:389-395`, `:811-850`), and a re-adjudication of the `atomization` row
against them;
(c) the D-row Q14/D116 single-rule constraint on the history-window row;
(d) an explicit **RESERVED-LINE check** — the inventory performs none, and while this system
is almost certainly clean (it computes a mean of edge tensions and broadcasts one opposition
gap), every sibling inventory in this train states the check and its result, and an absent
check is not a negative result.
