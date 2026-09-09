# FieldDerivativeSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `FieldDerivativeSystem` (`src/babylon/engine/systems/field_derivative.py`,
458 lines, tick position 20.0) computes a spatial gradient on every graph edge, an unweighted graph
Laplacian plus two temporal derivatives (df/dt, d2f/dt2) on every `SOCIAL_CLASS` node, picks a
graph-level "principal field" by max |df/dt|, and re-serializes everything into one `field_stack`
graph attribute so the Python facade's lossy `WorldState↔BabylonGraph` round trip survives. Two of
its four phases write data that has **no storage location in BSL at all today**: Phase 1's per-edge
`field_gradients` needs the still-unbuilt dyadic edge-attribute lane (Slice 2) *and* the substrate's
edge storage is one bare `f64` "strength" scalar with no attribute slots regardless (`update-edge`
refuses at evaluation for exactly this reason); Phase 3/4's graph-level `principal_field`/
`field_stack` writes have no verb in the closed seven-verb structural algebra at all — a gap this
system is named in by number (`reports/bsl-gap-analysis-2026-08-10.md`'s Q12, alongside
ControlRatio/Metabolism/Contradiction/WealthDistribution/TickDynamics) and re-confirmed live on
current dev by reading `tick.rs` directly. Phase 2 (Laplacian + temporal derivatives) is the one
phase expressible with today's landed query lane, but only after collapsing the Python's dynamic,
field-name-agnostic design down to the two field names production ever actually computes, and only
with a flagged double-counting risk when the same node pair carries two edge types (a real,
canonical-scenario-witnessed topology). Zero libm hazards; one distinct `EventType` emission
(`PRINCIPAL_CONTRADICTION_SHIFT`, a WS1 ledger row, consumed only by the narrative layer).

**Verdict: BLOCKED — on two independent, stacked gaps: (1) the dyadic edge-attribute lane (query
slice 2) compounded by a substrate storage gap that landing slice 2 alone would NOT fix, and (2) an
entirely unscheduled gap — no BSL verb writes a graph-level attribute at all. Phase 2 (Laplacian +
temporal derivatives) is PORTABLE WITH D-RECORDS in isolation, once a field-name collapse and a
history ring-buffer content model are declared.**

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/field_derivative.py` | 458 | **The target.** `FieldDerivativeSystem`, one `step()` orchestrating four phases plus a preamble. Read completely, line by line. |
| `src/babylon/engine/systems/contradiction_field.py` | 260 | **`ContradictionFieldSystem` @19.0, the same-tick prior writer.** Writes `SOCIAL_CLASS.contradiction_fields` and appends to `persistent_data["contradiction_history"]` *before* this system runs the same tick (`step()` at 70-89, `_step_from_oppositions` at 153-243 — the production E0 path, no `field_registry` wired). `_OPPOSITION_FIELD_NAMES = ("exploitation", "atomization")` (line 47) is the closed set field_derivative.py's own field-name discovery reduces to in production (§2, §6). |
| `src/babylon/engine/field_registry.py` | ~300 | `FieldRegistryProtocol`/`DefaultFieldRegistry` — the extensible-field-name mechanism `services.field_registry` would use. **Never wired in production** (`engine/services.py:196` defaults it `None`; grep-confirmed zero non-test instantiation of `DefaultFieldRegistry` anywhere in `src/`). Only test files (`tests/unit/engine/test_field_registry.py`, `tests/unit/engine/systems/test_contradiction_field_system.py`, `tests/unit/engine/systems/test_field_derivative_system.py`, `tests/integration/test_field_topology_integration.py`) construct one. Out of scope for a port of the *production* behavior — noted, not transcribed. |
| `src/babylon/formulas/curvature.py` | 236 | `compute_ollivier_ricci` (FR-005, Ollivier-Ricci curvature via `scipy.optimize.linprog`) — declared in the same Feature-002 spec family and re-exported from `formulas/__init__.py:78,247`, but **zero call sites in `engine/systems/*.py`** (grep-confirmed) — dead code relative to this system, same "not exercised" disposition as Territory's `territory_diagnostics.py`. Not read further; out of scope. |
| `src/babylon/config/defines/consciousness.py` | lines 272-332 (`ContradictionFieldDefines`) | Coefficient source for the whole Feature-002 stack: `field_min`(0.0)/`field_max`(10.0)/`history_window`(3, `[2,10]`)/`curvature_alpha`(0.5, `(0,1]`)/`co_optive_suppression_rate`(1.0, `[0,1]`)/`latent_release_multiplier`(1.5, `[1,5]`)/`default_transition_priority`(0, `>=0`). **None of these are read by `field_derivative.py`** — `field_min`/`field_max` are consumed by @19; `co_optive_suppression_rate`/`latent_release_multiplier`/`default_transition_priority` by @21 (EdgeTransitionSystem); `history_window` and `curvature_alpha` are read by **nothing anywhere in `src/`** (grep-confirmed) — dead defines (§2e, §6). |
| `src/babylon/data/defines.yaml` | `contradiction_field:` block, lines 399-408 | Player-editable values for the above — same "none read by this system" finding applies. |
| `src/babylon/models/world_state.py` | `field_stack`/`principal_field` fields 555-585; `_write_field_stack_graph_attrs` 801-829; `_restamp_field_stack` 831-876; `from_graph` reconstruction 942-951; `SOCIAL_CLASS_COMPUTED_FIELDS` 59-92; `_reconstruct_relationships` 357-393 | **The round-trip carrier this system's Phase 4 exists to feed.** `field_stack`/`principal_field` ARE declared `WorldState` Pydantic fields (loosely `dict[str, Any]`) — unlike the per-node/per-edge attrs, which are graph-only and explicitly excluded from reconstruction (`contradiction_fields`/`field_derivatives` in `SOCIAL_CLASS_COMPUTED_FIELDS`; `field_gradients` simply absent from `_reconstruct_relationships`'s field whitelist, so it is silently dropped — the documented "graph round-trip loses data" gotcha). `_restamp_field_stack` re-stamps them from the carrier every `to_graph()` call. |
| `src/babylon/engine/simulation_engine.py` | `_SYSTEM_CLASSES` 328-363; `step()` facade 503-611 | Confirms tick position 20.0 (§5) and the every-tick `state.to_graph() → run_tick → WorldState.from_graph()` round trip (552,572,606) that makes Phase 4's carrier necessary; `persistent_context` threading (556,566-568,577-580) that keeps `contradiction_history`/`_previous_principal_field` alive across ticks **only when the caller reuses the same dict** (§5's web-bridge dormancy nuance). |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase._get_persistent_data` (198-202, used, line 78). `_write_clamped`/`_publish` **not used** — this system calls `services.event_bus.publish(...)` directly (line 444), same non-use-of-the-shorthand pattern the Solidarity inventory already recorded as harmless. |
| `src/babylon/kernel/graph_protocol.py` | `query_nodes` 258-276; `query_edges` 278-298; `get_node` 77-86; `update_node` 88-98; `update_edge` 152-170; `get_graph_attr` 350-363; `set_graph_attr` 365-372 | `GraphProtocol` signatures this system calls. Note `query_edges()` (Phase 1 line 137, `_collect_neighbor_fields` line 269, `_build_field_stack` line 353) is called with **no `edge_type` filter anywhere** — the only system in the field-topology family with this shape (`ContradictionFieldSystem`'s `_build_tension_index` filters by `_FIELD_EDGE_TYPES`; this system scans every edge of every type). |
| `src/babylon/topology/graph.py` | `update_node` 660-670; `update_edge` 690-702; `get_graph_attr`/`set_graph_attr` 892-898 | Concrete `BabylonGraph`. All three are plain dict merges/lookups with **no type coercion, no quantization, no attribute-count limit** — the Python substrate accepts an arbitrary-shape `field_gradients` dict on an edge or a nested `principal_field` dict as a graph attr without complaint; the BSL substrate (§6) accepts neither. |
| `src/babylon/models/graph.py` | `GraphNode` 91-145; `GraphEdge` 147-185 | Frozen Pydantic wire types `query_nodes`/`query_edges`/`get_node` return; `.attributes: dict[str, Any]` on both. |
| `src/babylon/engine/context.py` | `TickContext` (114 lines) | `context.tick` (read, line 84); `context.persistent_data` (`_get_persistent_data`, line 78) — the only non-graph input this system reads. No `displacement_mode`-style override exists for this system. |
| `src/babylon/models/enums/topology.py` | `NodeType.SOCIAL_CLASS`; `EdgeType` 78-127 (24 members) | The one `NodeType` this system ever queries (`query_nodes(node_type=NodeType.SOCIAL_CLASS)`, 3 call sites: 120,183,339,395); the 24-member `EdgeType` enum that Phase 1/`_collect_neighbor_fields` deliberately does NOT filter by (§6's enumeration-burden finding). |
| `src/babylon/models/enums/events.py` | line 101 | `EventType.PRINCIPAL_CONTRADICTION_SHIFT = "principal_contradiction_shift"` — the system's one emission. |
| `src/babylon/models/events/field_payloads.py` | `PrincipalContradictionShiftEvent` 17-32 | Typed Pydantic event schema (`previous_field: str \| None`, `new_field: str`, `max_abs_df_dt: float`). Docstring cites `field_derivative.py:362-373` — **stale** relative to current line numbers (443-454 today); a minor, harmless cross-reference drift, noted per port-as-is honesty. |
| `src/babylon/game/chronicle_adapter.py` (250); `src/babylon/models/event_severity.py` (340,1001); `src/babylon/engine/event_builders.py` (389-395) | — | Downstream narrative/severity-classification consumers of `PRINCIPAL_CONTRADICTION_SHIFT` — all outside `engine/systems/*.py` (§5). |
| `src/babylon/sentinels/vocabulary/registry.py` | `EXTRA_STAMPABLE_ATTRIBUTES[SOCIAL_CLASS]` 201-211 | Confirms `contradiction_fields`/`field_derivatives` are sentinel-registered, legitimate graph-only (non-Pydantic-model) node attributes, each naming this system/`contradiction_field.py` as writer. |
| `src/babylon/sentinels/seam/registry.py` | `FIELD_STATE` scope, lines 1579-1700ish | Documents the web-bridge "altitude gap" this system's Phase 4 was built to close, and a live residual gap: on the **web bridge specifically**, `df_dt`/`principal_field` stay `DECLARED_CONDITIONAL` (not `MUST_BE_LIVE`) because `resolve_tick`'s `persistent_context` is a fresh `{}` per HTTP call — `contradiction_history` never accumulates across web ticks — independent of the round-trip carry (§5). |
| `web/game/engine_bridge.py` | `get_field_state` 4729; `_build_field_state_nodes` 1636; `_build_field_state_edges` 1700 | Legacy web-bridge reader of this system's outputs (Amendment V: web is legacy, its failures don't gate work) — out of scope for the port itself, cited only for the web-dormancy nuance above. |

**Reference BSL/Rust surface read for the query-lane and storage questions** (all read in full for
the cited ranges): `rust/crates/babylon-bsl/src/evaluator.rs` lines 464-527 (`EFFECT_POSITION_ONLY`/
`SERVED_QUERY_HEADS`), 486-512 (`UNSERVED_EXPRESSION_HEADS`), 2256-2267 (`EVALUATOR_SERVED`, the
authoritative landed-heads list); `rust/crates/babylon-bsl/src/structural_verbs.rs` lines 1-49
(module doc) and 385-402 (`update-edge`/`update-hyperedge` refusal, verbatim error text);
`rust/crates/babylon-bsl/src/declarations.rs` lines 44-82 (`RESERVED_FORM_TAGS`, the closed §2.8
verb vocabulary — confirms no `update-graph`/`set-graph-attr` verb exists at all) and line 110
(`DECLARABLE_INTRINSICS`); `rust/crates/babylon-bsl/src/tick.rs` lines 159,536 (`subject_type_of`,
called unconditionally by `run_tick`, never reads `loaded.domain`); `rust/crates/babylon-bsl/src/
domain.rs` lines 200-214,288 (`RuleDomain::Graph`/`resolve_domain` — load-time only); `docs/
reference/bsl-language.rst` §2.5 (bind-src table, `:out`/`:in`/`:any` at lines 571-573), §2.6
(query grammar, `<fold>`/`<neighbors>` productions, lines 942-951,1181-1186); `reports/bsl-gap-
analysis-2026-08-10.md` Q12 (lines 379-391, names this system explicitly); `reports/metabolism-
port-assessment-2026-08-11.md` §4(b) (lines 295-353, re-confirms the Q12 execution-engine gap
against `tick.rs` directly, on the closest sibling system to hit this exact gap).

## 2. COMPUTATION CATALOG (execution order, `field_derivative.py:53-103`, `step()` body)

### Phase 0 — Field-name discovery + early exit (`field_derivative.py:67-84`)
- **(a)** Determine which contradiction-field names exist this tick. Production never wires a
  `field_registry`, so it derives the set from whatever `ContradictionFieldSystem` @19 already wrote
  onto `SOCIAL_CLASS` nodes this same tick, rather than early-returning (the "E0" repoint documented
  in both systems' module docstrings). If no field names exist at all, the whole system is a no-op.
- **(b)** `registry = services.field_registry` (70) → `if registry is not None: field_names =
  registry.get_field_names()` (test-only path) `else: field_names = _discover_field_names(graph)`
  (71-74) → `if not field_names: return` (75-76, **before `persistent_data`/`tick` are even read**).
  `_discover_field_names` (106-124): `names: set[str] = set()`; for every `SOCIAL_CLASS` node, union
  `contradiction_fields.keys()` into `names`; return `sorted(names)`. In production this is
  *provably* always `["atomization", "exploitation"]` (alphabetical), since `_step_from_oppositions`
  (contradiction_field.py:153-202) unconditionally writes exactly `_OPPOSITION_FIELD_NAMES =
  ("exploitation", "atomization")` (contradiction_field.py:47,188-191) to every social_class node it
  touches — a closed, two-member set on every live production code path (§6's central finding).
- **(c) Reads:** `services.field_registry` (always `None` in production); `SOCIAL_CLASS.
  contradiction_fields.keys()` across every social_class node.
- **(d) Writes:** none.
- **(e) Defines:** none.
- **(f) Events:** none.

### Phase 1 — Spatial gradients on edges (`_compute_edge_gradients`, field_derivative.py:127-162)
- **(a)** For every edge in the graph, of any type, if both endpoints carry `contradiction_fields`,
  compute `target − source` for each field name and write the resulting dict to the edge.
- **(b)** `for edge in graph.query_edges():` (137, **unfiltered by `edge_type`** — the only unfiltered
  full-edge scan in this system, and the only one in the whole field-topology family) →
  `src_fields = src_node.attributes.get("contradiction_fields", {})`, same for `tgt_fields` (144-145)
  → `if not src_fields or not tgt_fields: continue` (148-149, both endpoints must be fielded — in
  practice both must be `SOCIAL_CLASS`, since only that node type ever carries the attribute) →
  `gradients[field_name] = tgt_val - src_val` for each field name (151-155) →
  `graph.update_edge(edge.source_id, edge.target_id, edge.edge_type, field_gradients=gradients)`
  (157-162).
- **(c) Reads:** `SOCIAL_CLASS.contradiction_fields` on both endpoints of every edge in the graph
  (no type filter).
- **(d) Writes:** `field_gradients` (dict `{field_name: float}`) on every edge whose both endpoints
  are fielded.
- **(e) Defines:** none.
- **(f) Events:** none.

### Phase 2 — Laplacian + temporal derivatives on nodes (`_compute_node_derivatives` +
`_collect_neighbor_fields`, field_derivative.py:165-296)
- **(a)** For each `SOCIAL_CLASS` node carrying `contradiction_fields`, compute a graph Laplacian
  (sum of weighted neighbor differences, over neighbors reached via *any* edge type in *either*
  direction — a genuinely undirected-graph view of a directed substrate) and two temporal
  derivatives from the 3-tick rolling history `ContradictionFieldSystem` @19 already appended to
  this same tick, before this system runs.
- **(b)** `neighbor_fields, edge_weights = _collect_neighbor_fields(graph, node_id, field_names,
  edge_weight_attr=edge_weight_attr)` (192-197) — `edge_weight_attr` is a keyword parameter of
  `_compute_node_derivatives` itself (line 169, default `None`), but **`step()` never passes it**
  (line 90 calls `_compute_node_derivatives(graph, field_names, history)` with exactly three
  positional arguments) — so in production `edge_weight_attr` is *always* `None`, meaning
  `_collect_neighbor_fields`'s weight branch (277-281: `float(edge.attributes.get(edge_weight_attr,
  1.0))` when set) never fires; every weight is unconditionally `1.0` (280). The weighted Laplacian
  is a real, tested (`tests/unit/infrastructure/test_weighted_laplacian.py`), but **entirely
  unreachable-from-production** feature — exercised only by white-box unit tests calling the
  module-private functions directly, never through `step()`.
  Laplacian: `laplacian = sum(w * (nv - my_val) for w, nv in zip(edge_weights, neighbor_vals,
  strict=True))` when `neighbor_vals` is non-empty (206-208), else `laplacian = 0.0` with a debug log
  (210-216) — **the nested `if not neighbor_vals:` guard at 211 inside the `else` branch is always
  true there** (a vestigial, always-satisfied condition — verbatim oddity, port-as-is, no behavioral
  effect: the log fires unconditionally whenever the Laplacian falls back to 0.0).
  Temporal: `df_dt = field_hist[-1] - field_hist[-2]` when `len(field_hist) >= 2` (223-225);
  `d2f_dt2 = field_hist[-1] - 2.0 * field_hist[-2] + field_hist[-3]` when `len(field_hist) >= 3`
  (227-229) — both `None` (an explicit Option, not a numeric default) below their history-length
  threshold.
  `graph.update_node(node_id, field_derivatives=field_derivatives)` (237), where
  `field_derivatives[field_name] = {"laplacian": ..., "df_dt": ..., "d2f_dt2": ...}` (231-235).
  `_collect_neighbor_fields` (240-296) itself: for every edge in the graph (again **unfiltered by
  type**, line 269), if either endpoint is `node_id`, record the *other* endpoint as a neighbor —
  **deduplicated by neighbor node id, first-encountered edge wins** (`if nid is not None and nid not
  in neighbor_weights:`, 276) — so a node pair connected by **two or more distinct edge types**
  contributes to the Laplacian **exactly once**, via whichever edge `query_edges()` iterates first
  (§4, §6's double-counting finding; this dedup IS observed on a canonical scenario, not a synthetic
  edge case — see §5).
- **(c) Reads:** `SOCIAL_CLASS.contradiction_fields` (self + every neighbor reached via any edge, any
  direction); `persistent_data["contradiction_history"][node_id][field_name]` (list, ≤3 entries,
  written earlier this same tick by `ContradictionFieldSystem` @19, contradiction_field.py:196-202/
  140-147 — the 3-entry cap is `_MAX_HISTORY_WINDOW = 3`, a **hardcoded module constant** in
  `contradiction_field.py:36`, not read from the `history_window` define at all, §2e's dead-define
  finding).
- **(d) Writes:** `SOCIAL_CLASS.field_derivatives` (dict of `{field_name: {laplacian, df_dt,
  d2f_dt2}}`).
- **(e) Defines:** none — the `2`/`3` thresholds in `len(field_hist) >= N` (223,227) are Python
  integer literals, not defines-driven, even though `contradiction_field.history_window` (default 3,
  domain `[2,10]`, defines.yaml:404) exists and *looks* like it should govern exactly this — it is
  read by **nothing anywhere in `src/`** (grep-confirmed). Changing `history_window` in
  `defines.yaml` today has zero runtime effect on either system.
- **(f) Events:** none.

### Phase 3 — Principal contradiction identification (`_identify_principal_contradiction`,
field_derivative.py:371-457)
- **(a)** Across all `SOCIAL_CLASS` nodes, find the field name with the largest per-node |df/dt|
  (ignoring nodes/fields where df/dt is still `None`), tie-broken by summed |df/dt| magnitude across
  all nodes, then by a hardcoded preference for the literal field name `"exploitation"`. Write the
  winner plus whether it changed since last tick to a **graph-level** attribute; if it changed and is
  non-null, emit an event.
- **(b)** Per-field aggregates, one pass over all social_class nodes: `field_max_abs_df_dt[field] =
  max(field_max_abs_df_dt[field], abs(df_dt))` (401-403, only updates the max, does not accumulate
  every value) and `field_total_magnitude[field] += abs(df_dt)` (404, unconditional accumulate) —
  both dicts pre-seeded `dict.fromkeys(field_names, 0.0)` (392-393). Selection: `sorted(field_names,
  key=lambda f: (field_max_abs_df_dt[f], field_total_magnitude[f], 1.0 if f == "exploitation" else
  0.0), reverse=True)` (411-419) — the tiebreak is a **literal string-equality special case baked
  into engine code**, not defines-driven or content-driven. Promotion guard: `if candidates and
  field_max_abs_df_dt[candidates[0]] > 0.0:` (421) — the winning field is only promoted to
  `principal_field` when its own max |df/dt| is strictly positive; otherwise `principal_field` stays
  `None` and `max_df_dt` stays `0.0` even though `candidates` is non-empty (i.e. the sort always
  produces a "winner" by tiebreak, but the winner is only *published* if it actually moved).
  `changed = principal_field != previous_principal` (427), reading
  `persistent_data.get("_previous_principal_field")` (426, an underscore-prefixed private
  bookkeeping key, same convention as `contradiction_field.py`'s `_field_previous_wealth`).
- **(c) Reads:** `SOCIAL_CLASS.field_derivatives[field]["df_dt"]` (all nodes, all field names,
  396-404); `persistent_data["_previous_principal_field"]` (426).
- **(d) Writes:** **graph-level** attribute `principal_field = {"field_name": str | None,
  "max_abs_df_dt": float, "changed": bool}` via `graph.set_graph_attr(...)` (433-440, **not** a node
  or edge attribute — §6's central second blocker); `persistent_data["_previous_principal_field"]`
  (457).
- **(e) Defines:** none.
- **(f) Events:** `EventType.PRINCIPAL_CONTRADICTION_SHIFT` (446), emitted only when `changed and
  principal_field is not None` (443), via `services.event_bus.publish(Event(...))` called
  **directly** rather than through `SystemBase._publish` (functionally identical, a style-only
  deviation, port-as-is). Payload: `{"previous_field", "new_field", "max_abs_df_dt"}` (447-453).

### Phase 4 — Field-stack snapshot (`_build_field_stack`, field_derivative.py:299-368, invoked
at line 103)
- **(a)** Re-reads back everything Phases 1-2 just wrote this tick (and whatever `ContradictionFieldSystem`
  @19 wrote before them) and assembles ONE deterministic, sorted, nested dict as the
  `WorldState.to_graph()`/`from_graph()` round-trip carrier — pure engine plumbing working around the
  Python facade's own graph-round-trip data loss, not new game-state computation (§6).
- **(b)** Per node: `entry["fields"] = {name: fields[name] for name in sorted(fields)}` (346),
  `entry["field_derivatives"] = {name: derivs[name] for name in sorted(derivs)}` (348) — included
  only if `fields` or `derivs` is non-empty (342-343, honest omission per Constitution III.11); node
  dict itself key-sorted (350). Per edge: **one row per (edge, field) pair** — `{"source", "target",
  "field", "gradient"}` (358-365), a fan-out/denormalization, sorted by `(source, target, field)`
  (366). `graph.set_graph_attr("field_stack", {"nodes": ..., "edges": ...})` (line 103) — always
  written whenever `step()` reaches this point (i.e. whenever `field_names` was non-empty at Phase 0),
  even when both collections end up empty (confirmed by `test_field_stack_omits_edge_without_
  gradients`, which still gets a present-but-empty-edges `field_stack`).
- **(c) Reads:** `SOCIAL_CLASS.contradiction_fields`/`field_derivatives` (all nodes, 339-341); every
  edge's `field_gradients` (no type filter, 353-354).
- **(d) Writes:** **graph-level** attribute `field_stack = {"nodes": {...}, "edges": [...]}` — same
  storage-location problem as Phase 3's write.
- **(e) Defines:** none.
- **(f) Events:** none.

**Events emitted by the whole system: exactly one distinct `EventType`** —
`PRINCIPAL_CONTRADICTION_SHIFT`. Grep-confirmed (`rg -n "EventType\." field_derivative.py`): the
single occurrence is Phase 3's line 446.

## 3. TYPE INVENTORY

Runtime storage note (same finding as Territory/Solidarity, re-verified here):
`BabylonGraph.update_node`/`update_edge`/`set_graph_attr` (`topology/graph.py:660-670`,`:690-702`,
`:896-898`) are plain dict writes with **no type coercion, no quantization, and no shape/arity
limit** — the Python side accepts an arbitrarily-nested `field_gradients` dict on an edge or a
`principal_field` dict as graph metadata without complaint. All in-tick arithmetic below is raw
Python `float`/`bool`/`str | None`.

| Attribute | Node/edge/graph scope | Python shape | Domain | Category |
|---|---|---|---|---|
| `contradiction_fields` | node, `SOCIAL_CLASS` | `dict[str, float]` (graph-only, not a `SocialClass` model field — `EXTRA_STAMPABLE_ATTRIBUTES`, registry.py:204) | keys: closed 2-member set in production (`{"atomization","exploitation"}`); values: `[0.0, 10.0]` (clamped by @19 against `field_min`/`field_max`) | **map-valued attribute**, values unit-interval-adjacent but scaled to 10, not 1 |
| `field_derivatives` | node, `SOCIAL_CLASS` | `dict[str, dict[str, float \| None]]` (graph-only, registry.py:205) | per-field sub-dict `{"laplacian": float, "df_dt": float \| None, "d2f_dt2": float \| None}` — laplacian/df_dt/d2f_dt2 all **unbounded reals** (sum/difference of `[0,10]`-bounded values times up to N unit weights) | **map-of-struct-valued attribute**; `df_dt`/`d2f_dt2` are genuine **Option types** (undefined-until-N-ticks, not a numeric default) |
| `field_gradients` | edge, any type | `dict[str, float]` (graph-only; **not** in `Relationship`'s reconstruction whitelist at all — silently dropped by `_reconstruct_relationships`, only survives via the Phase-4 carrier) | unbounded real (difference of two `[0,10]`-bounded values) | **map-valued edge attribute** |
| `principal_field` | **graph-level** (`WorldState.principal_field`, a declared but loosely-typed `dict[str, Any]` model field, world_state.py:573-585) | `{"field_name": str \| None, "max_abs_df_dt": float, "changed": bool}` | `field_name` closed to the 2-member production set or `None`; `max_abs_df_dt` unbounded-above real ≥ 0 | **graph-scope struct**, not per-node/per-edge at all |
| `field_stack` | **graph-level** (`WorldState.field_stack`, world_state.py:555-571) | `{"nodes": dict, "edges": list}` | — | **graph-scope nested collection**, pure round-trip plumbing |
| `_previous_principal_field` | `persistent_data` key (not graph state at all) | `str \| None` | closed 2-member set or `None` | engine-only cross-tick bookkeeping |
| `contradiction_history` | `persistent_data` key (not graph state at all) | `dict[node_id, dict[field_name, list[float]]]`, lists ≤3 entries | list values `[0.0, 10.0]` | **cross-tick rolling window**, node-keyed but stored entirely off-graph |
| `field_min`/`field_max`/`history_window`/`curvature_alpha`/`co_optive_suppression_rate`/`latent_release_multiplier`/`default_transition_priority` (defines) | — | `float`/`int` | see FILE MAP row | **none read by this system** — see §2e/§6 |

**Enum discriminant flag: none of this system's own attributes.** Unlike Territory, this system
reads/writes no `deffield`-eligible enum-typed scalar of its own — `contradiction_fields`' keys are
*field names*, not an enum operand, and there is no `deffield enum` for a field NAME anyway (§6).

**Map/struct-valued attribute flag — the genuinely new structural finding this system contributes,
worse than Solidarity's nested-`ideology` case.** Solidarity's `ideology` was one fixed-shape nested
`BaseModel` (3 named sub-fields), flattening cleanly to 3 independent `deffield`s. Here, `deffield`'s
closed scalar-only vocabulary (`int, bool, currency, probability, intensity, coefficient, enum`) has
to represent a `dict[field_name, dict[metric_name, float | None]]` — a genuinely two-level,
*variable-keyed* structure. The field-name axis collapses cleanly (production is provably a closed
2-member set — §6), so the flattening becomes `2 fields × 3 metrics = 6` independent scalar
`deffield`s (`social-class/exploitation-laplacian`, `social-class/exploitation-df-dt`, …,
`social-class/atomization-d2f-dt2`) — BUT `df_dt`/`d2f_dt2`'s `float | None` shape has **no
BSL-native representation at all**: `deffield`'s type vocabulary has no Option/nullable variant, so
"undefined until 2 (or 3) ticks of history exist" needs either a sentinel value (a content-modeling
deviation with its own honesty cost — a magic float claiming to mean "no data yet" is exactly the
kind of fabricated-value smell Constitution III.11 exists to forbid) or a companion `bool`
`deffield` per Optional metric (`social-class/exploitation-df-dt-defined`) read before every use — a
genuinely new representational pattern with no precedent in any inventoried system so far, requiring
its own D-record.

**Graph-level (not node/edge) attribute flag — the system's central structural finding, distinct
from anything Territory/Solidarity/Metabolism needed.** `principal_field` and `field_stack` are
`WorldState` model fields, not per-element attributes — they exist because `graph.set_graph_attr`
targets the graph as a whole. `deffield`'s `<domain>` production (bsl-language.rst §2.3) is always
exactly one `NodeType` member (D43, cited by the sibling Survival inventory for an analogous
cross-type problem) — there is no `deffield`-level graph-scope declaration at all, and even the
*rule*-scope `(domain :graph)` construct (§2.3, meant for exactly this "fires once per tick, not
once per node" shape) is load-time-only (§6).

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `float`). Grep-confirmed zero `exp`/`log`/`sigmoid`/`pow`/
`math.`/`numpy`/`np.` calls anywhere in `field_derivative.py` or `contradiction_field.py` — **this
system has zero libm-nondeterminism hazard**, matching Territory and Solidarity. (`formulas/
curvature.py` uses `scipy.optimize.linprog`, but it is dead code relative to this system — §1 — so
it contributes no hazard to the port.) Shapes, in execution order:

1. **Subtractive difference (gradient):** `tgt_val - src_val` (field_derivative.py:155) — one
   subtract, per (edge, field) pair.
2. **Weighted-sum accumulation (Laplacian):** `sum(w * (nv - my_val) for w, nv in zip(edge_weights,
   neighbor_vals, strict=True))` (206-208) — a multiply-then-subtract per neighbor, reduced via
   Python's `sum()`. In production `w` is always `1.0` (§2), so this degenerates to a plain sum of
   differences — but the **summation order is exactly the neighbor-dict insertion order**
   (`neighbor_weights`, a plain dict populated by iterating `query_edges()`, §2's dedup finding) —
   a real bit-for-bit reproducibility contract for any reimplementation, not merely a style note,
   since floating-point addition is not associative.
3. **First temporal derivative:** `field_hist[-1] - field_hist[-2]` (225) — one subtract.
4. **Second temporal derivative:** `field_hist[-1] - 2.0 * field_hist[-2] + field_hist[-3]` (229) —
   one multiply, two adds/subtracts by Python operator precedence (`(a - (2.0*b)) + c`). Bare
   **non-integer literal `2.0`** — the same "no bare non-integer literal" BSL parser constraint
   Territory/Solidarity flagged (needs a `c`-suffixed const or the Real-zero-promotion idiom), here
   on an integer-valued-but-float-typed literal specifically.
5. **Magnitude + running max/sum (principal-field selection):** `abs_val = abs(df_dt)` (401);
   `if abs_val > field_max_abs_df_dt[field]: field_max_abs_df_dt[field] = abs_val` (402-403, a
   running max, order-independent); `field_total_magnitude[field] += abs_val` (404, a running sum,
   order-DEPENDENT in the same bit-for-bit sense as item 2 — iteration is `query_nodes(SOCIAL_CLASS)`
   order). `abs` is a plain unary op — not a declared BSL intrinsic (grep-confirmed zero `"abs"`
   hits anywhere in `rust/crates/babylon-bsl/src/*.rs`), but trivially expressible via `if x < 0
   then (sub 0 x) else x` (nested-`if`, same idiom Territory used for `min`/`max`), no new primitive
   needed.
6. **Bare-literal tiebreak:** `1.0 if f == "exploitation" else 0.0` (416) — two more bare non-integer
   literals, plus a **hardcoded string-equality special case baked into engine code** (§6) rather
   than a defines-driven or content-driven rule.
7. **Threshold comparison:** `field_max_abs_df_dt[candidates[0]] > 0.0` (421) — plain `>`, bare `0.0`
   literal.
8. **Real→Int demotions: none.** Grep-confirmed zero `int(...)` casts anywhere in `field_derivative.py`
   — a favorable contrast with Territory's two `floor`-class demotions.
9. **Currency-mixing multiplies: none.** No operand anywhere in this system is `Currency`-typed in
   either the Python or BSL sense — every value is a plain `[0,10]`-bounded "field" float or an
   unbounded derivative real. The D-1-class scale-op hazard (Territory's `rent_spike_multiplier`,
   Metabolism's `entropy_factor`) does not arise here.
10. **Weighted-edge cast (dead in production):** `float(edge.attributes.get(edge_weight_attr, 1.0))`
    (278) — a cast plus a bare `1.0` default, but the whole branch is unreachable from `step()` (§2).

**Order-sensitivity + double-counting compound finding (the system's most consequential float-op
oddity, verified against a canonical scenario, not hypothesized).** The Laplacian's neighbor
collection (`_collect_neighbor_fields`, field_derivative.py:240-296) deduplicates by neighbor node
id — a node pair connected by **two distinct edge types** contributes to the Laplacian exactly
**once**, via whichever edge `query_edges()`'s insertion-order iteration reaches first. This is not
a hypothetical edge case: `create_imperial_circuit_scenario` (`engine/scenarios/_legacy.py:255-460`,
the *default* qa:regression scenario) seeds **both** `TRIBUTE: P_c(C002) → C_b(C003)`
(`_legacy.py:416-424`) **and** `CLIENT_STATE: C_b(C003) → P_c(C002)` (`_legacy.py:436-445`, the
reverse direction, same unordered node pair) — so on this exact
canonical scenario, `C_b`'s and `P_c`'s Laplacians each count the other exactly once despite two
distinct edges connecting them, and in the unweighted production case (weight always `1.0`, §2) this
dedup happens to be the semantically *correct* simple-graph Laplacian (each adjacent pair contributes
once regardless of edge multiplicity) — not a bug, but a behavior any BSL reformulation must
reproduce exactly, and a naive "one `fold sum` per `EdgeType`, then add the sums" reformulation
(§6's proposed workaround for the missing any-type `neighbors`) would **double-count** this exact
pair, a genuine, canonical-scenario-witnessed divergence risk, not a corner case to wave away.

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 20.0** (`field_derivative.py:47`), confirmed against `_SYSTEM_CLASSES`
  (`simulation_engine.py:328-363`) and each file's own `position: ClassVar[float]`: `... →
  ContradictionSystem(18.0) → ContradictionFieldSystem(19.0) → FieldDerivativeSystem(20.0) →
  CollapseTransitionSystem(20.5, `collapse_transition.py:54`) → EdgeTransitionSystem(21.0,
  `edge_transition/_legacy.py:574`) → WealthDistributionSystem(21.5) → EpistemicHorizonSystem(22.0,
  last)`. `tests/unit/engine/test_system_order.py:93,182-198` pins the 34-system total and this
  system's presence, but its own `test_contradiction_runs_last` (172-180) is a weak assertion (only
  checks `ContradictionSystem` is registered before the end of the list, not a genuine ordering
  contract against `ContradictionFieldSystem`/`FieldDerivativeSystem` specifically) — noted as a
  test-coverage gap, not a production defect.
- **Reads from a same-tick prior system — the system's defining channel:**
  `SOCIAL_CLASS.contradiction_fields` and `persistent_data["contradiction_history"]` are written by
  `ContradictionFieldSystem` @19.0 **one position earlier the same tick** (contradiction_field.py:
  138,195 and 140-147/196-202) — every Phase of this system depends on that same-tick write; there
  is no cross-tick lag anywhere in this system's own reads (unlike, e.g., Metabolism's `context.
  persistent_data["balkanization.metabolic_impact_by_territory"]` one-tick lag).
- **Writes consumed later this tick / downstream ticks:**
  - `SOCIAL_CLASS.contradiction_fields`/`field_derivatives` and edge `field_gradients` — read by
    `EdgeTransitionSystem` @21.0, **the same tick, one position later**
    (`engine/systems/edge_transition/_legacy.py:507` reads `contradiction_fields` for its `"value"`
    metric predicates; `:510` reads `field_derivatives` for its `"df_dt"`/`"d2f_dt2"`/`"laplacian"`
    metric predicates, driving `EdgeMode` compound-predicate transitions; `:732-734` reads
    `field_derivatives` again inside `_accumulate_co_optive_latent`, multiplying `df_dt` by
    `services.defines.contradiction_field.co_optive_suppression_rate` to accumulate suppressed
    latent contradiction on `CO_OPTIVE` edges). This is the system's single most load-bearing
    downstream channel — a genuine, real, same-tick consumer, not a dormant one.
  - `principal_field`/`field_stack` (graph-level) — re-stamped by `WorldState.to_graph()`/
    `_restamp_field_stack` every subsequent tick's round trip (world_state.py:780,831-876), and read
    by the legacy web bridge (`web/game/engine_bridge.py::get_field_state`) — no `engine/systems/
    *.py` reads either graph attr (grep-confirmed).
  - `EventType.PRINCIPAL_CONTRADICTION_SHIFT` — consumed only by `game/chronicle_adapter.py`,
    `models/event_severity.py`, `engine/event_builders.py` (narrative/severity-classification layer,
    §1); no `engine/systems/*.py` branches on it.
- **Context/service usage with no BSL equivalent:** `context.tick` (field_derivative.py:84) — used
  only to stamp the emitted event's `tick` field; trivial, the tick number is an ambient rule-
  evaluation input already. `services.field_registry` — production-dead (§1).
- **A cross-tick fragility not visible from the graph alone.** `contradiction_history` and
  `_previous_principal_field` live entirely in `context.persistent_data`, threaded across ticks via
  the **caller-supplied** `persistent_context` dict (`simulation_engine.py:566-568,577-580`) — NOT
  via the graph round trip at all. The headless runner and `tools/regression_test.py`'s tick loop
  both reuse one `persistent_context` dict across the whole run (confirmed: `tools/regression_test.
  py:1023`, `persistent_context: dict[str, Any] = {}` created once, outside the `for tick in
  range(...)` loop), so history accumulates correctly there — but the **web bridge's `resolve_tick`
  constructs a fresh `{}` per HTTP call** (`sentinels/seam/registry.py:1640-1646`'s own documented
  finding, independent of this inventory), so `df_dt`/`principal_field` never actually go live on
  the web bridge in practice, despite the Phase-4 round-trip carrier fixing the *graph*-side of the
  altitude gap. Out of scope for the port itself (a Python-web-specific plumbing gap), noted for
  completeness since it is exactly the kind of "declared conditional, not a fabricated live value"
  distinction Constitution III.11 cares about.
- **DORMANCY on canonical scenarios — this system is LIVE, not dormant, unlike Territory.**
  `ContradictionSystem` @18.0 unconditionally writes `tension` onto every `EXPLOITATION`/`WAGES`/
  `TENANCY` edge every tick (`engine/systems/contradiction.py:215`) and the graph-level
  `opposition_states` attr every tick — no scenario-dependent gating. Every one of the 11
  `tools/regression_scenarios.py` canonical scenarios seeds `SOCIAL_CLASS` nodes connected by
  `EXPLOITATION`/`WAGES`/`TRIBUTE`/`CLIENT_STATE`/`SOLIDARITY` edges (verified directly in
  `create_imperial_circuit_scenario`, the default scenario, `engine/scenarios/_legacy.py:255-410`) —
  so `ContradictionFieldSystem` @19 always produces a non-empty `contradiction_fields` set, and this
  system's Phases 1-2 fire on every tick of every canonical scenario from tick 1 onward (`df_dt`
  needs ≥2 ticks, `d2f_dt2` needs ≥3 — trivially satisfied on any multi-tick run; the canonical run
  status tracks ticks well past 500). This is a materially different dormancy profile from Territory
  (whose ADJACENCY-dependent phases are structurally dormant on every canonical scenario) — a port's
  conformance fixtures for Phases 1-2 CAN plausibly be harvested from the canonical estate, not only
  hand-built. Phases 3-4 (graph-level writes) fire on the same live schedule but are irrelevant to
  the canonical byte-gate specifically, since `tools/regression_test.py::graph_content_hash` hashes
  node/edge attributes, not `WorldState.field_stack`/`principal_field` (**UNVERIFIED beyond a
  targeted read of `graph_content_hash`'s own field list — not re-run here, read-only mandate** —
  worth a follow-up check before assuming byte-gate coverage of Phase 3/4's outputs one way or the
  other).

## 6. BLOCKER ASSESSMENT

Adjudicated directly against the current dev tree (`rust/crates/babylon-bsl/src/evaluator.rs`,
`structural_verbs.rs`, `declarations.rs`, `tick.rs`, `domain.rs`; `docs/reference/bsl-language.rst`
§2.3/§2.5/§2.6), verified live in this session (§1's citation list), and cross-checked against the
two closest sibling findings on dev: `reports/bsl-gap-analysis-2026-08-10.md` Q12 (names this system
explicitly) and `reports/metabolism-port-assessment-2026-08-11.md` §4(b) (re-confirms Q12's
execution-engine half against `tick.rs` directly, one day before this inventory, for the closest
sibling system to hit the same gap).

| Computation | Verdict | Detail |
|---|---|---|
| Phase 0 — field-name discovery (field_derivative.py:67-84) | **PORTABLE WITH D-RECORD (favorable collapse)** | Production is *provably* a closed 2-member field-name set (`_OPPOSITION_FIELD_NAMES`, contradiction_field.py:47) — the Python's dynamic, registry-driven, field-name-agnostic architecture is a generality never exercised outside tests (`DefaultFieldRegistry` is never wired in production, §1). The pack should NOT attempt a dynamic dict-of-fields representation (no BSL primitive for that anyway, §3) — it should declare two hardcoded, unrolled computations for `exploitation` and `atomization` by name, exactly the "provably uniform on every real code path" reasoning class Metabolism's D-2 and Territory's `displacement_mode` D-record already established. Risk, named not hidden: this collapse is contingent on `field_registry` staying permanently unwired in production — the same category of risk as Territory's `displacement_mode` override. |
| Phase 1 — edge gradients + `field_gradients` write (field_derivative.py:127-162) | **BLOCKED — triple-stacked, on the dyadic edge-attribute lane AND a deeper storage gap** | (1) Selecting *any* edge at all as a first-class value needs the `edges`/`edge-between` query heads, explicitly `UNSERVED_EXPRESSION_HEADS` tagged `"slice 2"` (evaluator.rs:504-505) — not built. There is no `neighbors`-based workaround: `neighbors` (landed) yields `NodeRef`s only, never an `EdgeRef` (bsl-language.rst §2.6's result-type table, same finding the Solidarity inventory already made for its own edge-resident datum). (2) Even granting slice 2, **`GraphSubstrate`'s edge storage is one bare `f64` "strength" scalar with no attribute slots at all** (structural_verbs.rs:16-26) — there is no room for a per-field `field_gradients` map regardless of query-lane status. (3) The write verb itself, `update-edge`, is grammar-recognized (D35, declarations.rs — confirming it is a genuine, named, closed-vocabulary member, not an unknown head) but **refuses at evaluation with a stated reason**: *"has no substrate storage: GraphSubstrate keys an edge to one f64 strength and gives a hyperedge no attributes at all. Widening that state widens the canonical state_hash field set, which is a declared Phase-2/substrate decision (Constitution III.7), never a silently-dropped write"* (structural_verbs.rs:387-398, verbatim). Landing slice 2 (reads) would NOT by itself unblock this phase — the write-side storage gap is independent and, per the verb's own comment, requires a separate, deliberately-escalated substrate-widening decision. |
| Phase 2 — Laplacian (`_compute_node_derivatives`'s unweighted sum, field_derivative.py:206-208, 240-296) | **PORTABLE WITH D-RECORDS (two, both named)** | The unweighted case (the only one ever live in production, §2) reformulates cleanly as `fold sum` over `(neighbors self EdgeType/X :any NodeType/SOCIAL_CLASS)` — `neighbors`, `fold`, and the `:any` direction flag are all landed (`SERVED_QUERY_HEADS`/`EVALUATOR_SERVED`, evaluator.rs:527,2256-2267; bsl-language.rst:571-573). Two D-records, both real content-modeling risk, not free: **(D-record A, edge-type enumeration)** `neighbors` takes exactly one `EdgeType` operand (bsl-language.rst §2.6's `<neighbors>` production, no wildcard) — the frozen Python's unfiltered any-type scan needs one `fold sum` per relevant `EdgeType`, added together (valid for a SUM reducer specifically, since sum distributes over a query union). **(D-record B, double-counting risk, §4)** that per-EdgeType decomposition **double-counts any node pair connected by two or more edge types simultaneously** — a real topology on the *default* canonical scenario (`TRIBUTE`+`CLIENT_STATE` both connecting `C_b`↔`P_c`, §4) — where the frozen Python's neighbor-id dedup counts the pair once. The pack must either accept a declared, D-recorded divergence (double-counted multi-edge pairs) or restrict the enumerated `EdgeType` set to one that is provably pairwise-disjoint-per-node-pair on every scenario the pack targets (fragile, scenario-topology-dependent, needs re-verification on every new scenario). Neither option is free; this is a genuine, previously-unnamed content-modeling gap. |
| Phase 2 — temporal derivatives `df_dt`/`d2f_dt2` (field_derivative.py:223-229) | **PORTABLE WITH D-RECORD (new representational pattern, no precedent)** | Requires (a) a per-field 3-slot ring buffer as flat `deffield`s (§3) with a same-rule, source-ordered shift-then-store effect sequence (`§2.8`'s "effects apply in source order" guarantee, structural_verbs.rs:35-36, makes a single rule's own internal shift-then-store safe without needing the still-open cross-RULE pre-state question) — a genuinely new pattern this system introduces, not exercised by any inventoried system so far; and (b) a representation for "undefined until N ticks" (`deffield` has no Option/nullable type, §3) — a companion `bool` "defined" flag per Optional metric, or an explicit sentinel value accepted as a declared deviation. Neither is blocked by any named slice, but both are un-precedented transcription decisions needing their own D-records before a pack can be written. |
| `field_derivatives` storage shape, `2 fields × 3 metrics = 6` flat `deffield`s (cross-cutting, §3) | **D-RECORD, mechanical once the two decisions above land** | Same class of decision as Solidarity's `ideology`-flattening — favorable in the same way (deletes the read-modify-write struct dance), but compounded here by the Option-representation problem those 6 fields include (`df_dt`/`d2f_dt2` × 2 fields = 4 of the 6 need the Optional workaround). |
| Phase 3 — max/tiebreak selection over the 2 fixed fields (field_derivative.py:392-421) | **PORTABLE (trivial), contingent on Phase 2 landing first** | Once field names are collapsed to 2 hardcoded scalars (Phase 0) and their per-field aggregate max/sum exist (each itself a `fold max`/`fold sum` over `nodes typed SOCIAL_CLASS` — landed), comparing exactly 2 precomputed tuples via nested `if`/`when` needs no new BSL machinery — `abs` is expressible via nested-`if` (§4), the literal `"exploitation"`-preference tiebreak becomes ordinary content (a hardcoded `if field = exploitation-tag`), not new code. This step is entirely gated by the storage question below, not by its own arithmetic. |
| Phase 3 — `principal_field` graph-level write (field_derivative.py:433-440) | **BLOCKED — no verb, and this system is named by number in the underlying spec gap** | Two independent, stacked reasons. **(1) Q12, named explicitly.** `reports/bsl-gap-analysis-2026-08-10.md`'s Q12 ("Rule domain declaration, including graph-scoped rules") lists exactly six systems: *"ControlRatio, Metabolism, FieldDerivative, Contradiction, WealthDistribution, TickDynamics"* and states *"Three of them perform exactly one graph-level check per tick — ControlRatio's four-phase state machine, Metabolism's overshoot check, FieldDerivative's principal-contradiction pick. Under per-node inference those would emit once per node"* (lines 379-391, verbatim) — this system is one of the three worked examples, not a marginal case. Re-verified live on current dev, not merely cited stale: `(domain :graph)` resolves cleanly at LOAD time (`domain.rs:200-214`, `RuleDomain::Graph`/`resolve_domain`), but `rust/crates/babylon-bsl/src/tick.rs::run_tick` **unconditionally** calls `subject_type_of(&loaded.bindings)` (tick.rs:536, definition at 159) and **never reads `loaded.domain` at all** — confirmed by direct grep (zero hits for `loaded.domain`/`RuleDomain::Graph` in `tick.rs`), independently reproducing `reports/metabolism-port-assessment-2026-08-11.md §4(b)`'s identical finding against the identical file one day earlier. A `(domain :graph)` rule today either fails at `subject_type_of` (no `:field` binding at rule scope) or is silently misinterpreted as an ordinary per-node rule. **(2) Even setting Q12 aside, entirely: no verb in the closed §2.8 structural-verb vocabulary targets anything but a node, edge, or hyperedge.** `declarations.rs:44-82`'s `RESERVED_FORM_TAGS` — the authoritative closed vocabulary — lists `update-node`/`update-edge`/`update-hyperedge` and nothing else in the update family; there is no `update-graph`/`set-graph-attr` verb at all, scheduled or unscheduled, on any of the four named query-evaluation slices. This is a gap Q12's own fix (wiring `loaded.domain` into `run_tick`) would not by itself close — Q12 is about a rule firing once-per-tick over graph scope, not about WHERE that rule's effect writes; storage for a graph-level attribute has no home in the verb algebra regardless. |
| Phase 3 — `PRINCIPAL_CONTRADICTION_SHIFT` event (field_derivative.py:443-454) | **PORTABLE WITH D-RECORD (WS1 ledger), contingent on the write above** | `emit` is landed and every pack uses it — mechanically servable in isolation. But the event's `changed` trigger condition depends on the graph-level `principal_field` write being computable at all (the blocker directly above), and per the CURRENT BSL surface `TickReport` carries no event log regardless, so this is a WS1 (#502) ledger row twice over — expressible in principle, unpinnable by any conformance golden today, and gated on an upstream blocker besides. |
| Phase 4 — `field_stack` snapshot + graph-attr write (field_derivative.py:299-368) | **NOT-A-PACK (engine plumbing, not game-state computation) — also independently BLOCKED on the same graph-level-write gap as Phase 3, were it in scope** | `_build_field_stack` computes no new game state; it exists solely to work around the Python facade's own `WorldState.to_graph()`/`from_graph()` data loss (§1, §5) — a Python-architecture-specific problem. A Rust/BSL engine need not do a lossy per-tick Pydantic-model↔graph round trip the way the Python facade does, so the question for the port is not "how do we express `field_stack` in BSL content" but "does this problem even recur in the Rust engine's own state representation at all" — a question for the engine/kernel layer, not a content-pack decision, and out of this inventory's scope to adjudicate. Recorded here so the eventual port does not mistake Phase 4 for a fourth game-rule phase requiring its own D-record; if it ever were judged in-scope as content, it would inherit the identical graph-level-write blocker Phase 3 already carries. |

**No RESERVED-LINE surface in this system.** `FieldDerivativeSystem` contains no doctrine-tree
content, no National Question parameters, and no outcome-definition logic — it is pure mechanical
graph-calculus (gradient/Laplacian/finite-difference derivatives) over values `ContradictionFieldSystem`
@19 already computed. Confirmed by full line-by-line read of `field_derivative.py`.

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_field_derivative_system.py` | 553 | **Primary conformance-oracle candidate.** Exhaustive per-phase coverage: gradient correctness (`test_gradient_is_target_minus_source`), Laplacian sum-of-differences + isolated-node zero case (EC-002), temporal derivatives across 1/2/3-tick sequences (EC-001's `None`-until-history-exists rule, explicit per-tick construction), the E0 no-registry production path (`test_derivatives_computed_without_registry_when_fields_present` — the one test that exercises production behavior end-to-end, not the test-only `DefaultFieldRegistry`), principal-contradiction selection + event emission, and the full `field_stack` snapshot contract (honest omission, sorted-key determinism, edge fan-out, node/edge round-trip verbatim-copy assertions) — 415-553 is effectively a second conformance-oracle section on its own for Phase 4. |
| `tests/unit/infrastructure/test_weighted_laplacian.py` | 163 | Unit coverage of the weighted-Laplacian branch — calls `_compute_node_derivatives`/`_collect_neighbor_fields` **directly** (module-private functions), the only place this codebase exercises `edge_weight_attr != None` at all. A conformance oracle for the *feature*, not for production `step()` behavior (§2/§6 — this branch is unreachable from `step()`). |
| `tests/integration/test_field_topology_integration.py` | 171 | Full 3-system pipeline (`ContradictionFieldSystem → FieldDerivativeSystem → EdgeTransitionSystem`) across multiple ticks on a stylized "Detroit metro" scenario — the closest thing to an end-to-end conformance vector for the same-tick cross-system channel (§5). Uses the **test-only** `DefaultFieldRegistry`, not the production E0 path — a port's own conformance fixtures should mirror the *production* path (`ServiceContainer.create()` with no `field_registry`) instead, per `test_derivatives_computed_without_registry_when_fields_present`'s pattern. |
| `tests/unit/models/test_graph_roundtrip.py` | lines ~854-1000 of 1182 | **Conformance oracle for Phase 4 specifically** (`TestFieldStackFacadeCarry`-shaped section, though not confirmed as an exact class name): round-trip idempotency, node/edge restamping from the carrier, absent-field-stack honest-empty-default, and the exclusion of `contradiction_fields`/`field_derivatives`/`field_gradients` from ordinary model reconstruction. Directly informs §6's Phase-4 NOT-A-PACK disposition — this is the file that proves the round-trip-loss problem the Phase 4 plumbing solves. |
| `tests/unit/projection/test_field_state.py` | 268 | Projection/read-model layer (`babylon.projection.field_state.project_field_state`, a fixture-fed pure function mirroring `web/game/engine_bridge.py::get_field_state`) — `observe()`-page-shaped, not engine-tick math; same category as Territory's `test_territory_anchor.py`. Out of scope as a computation oracle. |
| `tests/unit/engine/test_system_order.py` | 300 | Pins the 34-system total and this system's registered name (line 93); the ordering assertion specific to this system's neighborhood is weak (§5's finding) — a real, if minor, test-coverage gap: no test directly asserts `ContradictionFieldSystem` precedes `FieldDerivativeSystem` precedes `EdgeTransitionSystem` by name. |
| `tests/unit/engine/systems/test_contradiction_field_system.py` | 432 | Tests the upstream writer (@19), not this system — but its E0-path tests (`test_derivatives...` naming convention, `services = ServiceContainer.create()  # no field_registry`) are the ones a port's conformance fixtures should model for the *pre-state* this system consumes. |

**qa:regression byte-gate coverage.** Per §5's live-not-dormant finding, `tools/regression_test.
py::graph_content_hash` hashes node/edge attributes on every canonical scenario, so Phases 1-2's
outputs (`contradiction_fields`-derived `field_gradients`/`field_derivatives`) are exercised and
byte-gated on every one of the 11 canonical scenarios from tick 1 onward — a materially better
starting position for conformance-fixture harvesting than Territory's ADJACENCY-gated phases.
**UNVERIFIED**: whether `graph_content_hash`'s field list also covers the graph-level `field_stack`/
`principal_field` attrs (§5) — not re-run here (read-only mandate); a port's Phase-1 acceptance
checklist should confirm this one way or the other before assuming either coverage or its absence.

## Adjudication (2026-08-12)

Adjudicated against the **current dev tree** (`9324482f`), read-only, with fresh anchors.
The double-counting finding in §4 — verified against the *default* canonical scenario, not
hypothesised — is the single best piece of work in this train, and the Phase-0 field-name
collapse is a model of the "provably uniform on every real code path" reasoning class. The
**BLOCKED verdict survives on Phase 1 alone**; Phase 3's blocker is withdrawn.

1. **CORRECTION — Phase 3's second leg ("no verb … regardless") misses §3.6, the ratified
   ruling that disposes of exactly this case, and the inventory never read it.** `grep` over
   this report returns **zero** occurrences of "3.6", "carrier" or "ceiling 1"; the reference
   list covers §2.3/§2.5/§2.6, `evaluator.rs`, `structural_verbs.rs`, `declarations.rs`,
   `tick.rs`, `domain.rs` and the Q12 gap report — everything except the chapter that answers
   the question. `docs/reference/bsl-language.rst:2650-2689` (draft ruling, Phase 1 review,
   R9 chapter C3) opens *"Graph-scope state is ordinary node state on a declared carrier node
   type"*, states *"The ruling adds no new grammar and no new storage class"*, specifies the
   mechanism as "an ordinary `deffield` owned by a **carrier node type** — a `NodeType`
   member whose manifest `:ceiling` is 1 — read with `(field-of (the NodeType/…) …)` and
   written with `(update-node (the NodeType/…) …)`" (`:2662-2668`), and then **explicitly
   records the rejected alternative so it is not re-proposed**: *"The alternative was a
   `:global` bind-src plus an `update-global` verb. It was rejected because it invents a
   second storage class … A closed verb set is a property worth more than the convenience of
   writing `update-global`"* (`:2678-2685`). The row's own observation — that
   `RESERVED_FORM_TAGS` contains no `update-graph`/`set-graph-attr` — is factually correct
   (verified, `declarations.rs:40-82`) and is *the ruling's intended consequence*, not
   evidence of a gap. The verb is `update-node`; the storage is node storage.

2. **CORRECTION — Q12 is verified fact but is not load-bearing for THIS system, because the
   carrier route discharges "fires exactly once per tick" without `(domain :graph)`.** The
   inventory's live re-verification is sound: `run_tick` unconditionally calls
   `subject_type_of(&loaded.bindings)` (`tick.rs:536`, definition at `:159-181`) and never
   reads `loaded.domain` — confirmed. But `subject_type_of` derives the subject `NodeType`
   from the `:field` binding namespace, and `run_tick` then iterates
   `graph.nodes(&subject_type)` (`tick.rs:538`). A rule whose bindings are all
   `field-stack/*` is therefore anchored on the carrier, and at `:ceiling 1` its subject
   population is **exactly one node** — which is what "once per tick" means operationally.
   The per-field aggregates Phase 3 needs are landed folds from that anchor:
   `(fold max (nodes NodeType/SOCIAL_CLASS) (if (< d 0) (- 0 d) d))` with `nodes` in
   `SERVED_QUERY_HEADS` (`evaluator.rs:527`) and `abs` expressible as the nested-`if` the
   inventory itself proposes in §4 item 5 (confirmed: `rg '\babs\b'` over
   `declarations.rs`/`intrinsic_host.rs` returns zero — `abs` is not an intrinsic and does
   not need to be). The carrier `NodeType` is content-declarable today via
   `(defvocabulary NodeType (…))` (`scenario.rs:389-395`, `load_defvocabulary` `:811-850`;
   landed at `content/scenarios/organization-foundation.bscn:41`). **Phase 3's verdict should
   read PORTABLE WITH D-RECORDS**, contingent on Phase 2 as the row already says — Q12 stays
   a real, correctly-verified gap in the language, just not this system's blocker.

3. **CORRECTION — Phase 3 carries the §3 Option gap forward on a second axis, and the row
   does not name it.** §3 raises the `float | None` representation problem sharply and
   correctly for `df_dt`/`d2f_dt2` ("a magic float claiming to mean 'no data yet' is exactly
   the kind of fabricated-value smell Constitution III.11 exists to forbid"), then does not
   carry it into Phase 3, where it recurs on a different axis: `principal_field["field_name"]`
   is `str | None` (`field_derivative.py:433-440`) and `changed` compares against
   `persistent_data["_previous_principal_field"]`, also `str | None` (`:426`). The two
   *named* field values are served by the landed enum lane (ADR195/196), but the **third,
   unset state** — reached whenever the promotion guard `field_max_abs_df_dt[candidates[0]] >
   0.0` (`:421`) fails, which is every tick before any field moves — has no representation:
   `deffield`'s vocabulary is seven scalar rows with no nullable variant
   (`bsl-language.rst:2373`), and `GraphSubstrate::node_attribute` is total-or-error
   (`substrate.rs:142`; absence is `E-EVAL-033`, "absence is not a value",
   `evaluator.rs:1281-1290`). So Phase 3's D-record slate is the §3 companion-`bool` pattern
   applied a third time, not the "trivial, needs no new BSL machinery" the row states.

4. **CONFIRMATION — Phase 1's blocker, and it is even stronger than stated.** The row says
   "GraphSubstrate's edge storage is one bare `f64` 'strength' scalar with no attribute
   slots." Verified, and worse: the full trait surface (`substrate.rs:80-248`) exposes **no
   reader for that strength either** — the only edge accessor is
   `fn edges(&self, edge_type: &str) -> Vec<(NodeId, NodeId)>` (`:166`), returning bare id
   pairs, and `fn add_edge(…, strength: f64, …)` (`:111-116`) is write-only. So Phase 1 is
   blocked on **read and write** by the same unscheduled substrate widening, and the row's
   conclusion — "landing slice 2 (reads) would NOT by itself unblock this phase" — holds for
   a stronger reason than it gives. The `update-edge` refusal text is confirmed verbatim at
   `structural_verbs.rs:387-398`, including its own framing as "a declared Phase-2/substrate
   decision (Constitution III.7)".

5. **CONFIRMATION — the double-counting finding, verified against the canonical scenario.**
   `engine/scenarios/_legacy.py:416-424` seeds `TRIBUTE: COMPRADOR_ID → CORE_BOURGEOISIE_ID`
   and `:436-445` seeds `CLIENT_STATE: CORE_BOURGEOISIE_ID → COMPRADOR_ID` — the same
   unordered node pair, two distinct edge types, in `create_imperial_circuit_scenario`, the
   default `qa:regression` scenario. `_collect_neighbor_fields`'s dedup
   (`field_derivative.py:276`, `if nid is not None and nid not in neighbor_weights`) counts
   the pair once. A per-`EdgeType` fold decomposition would count it twice. This is a real,
   witnessed divergence risk on the default scenario, exactly as §4 and D-record B state, and
   it is the finding a naive reformulation would have shipped as a silent bug.

6. **CONFIRMATION, and §5/§7's UNVERIFIED flag RESOLVED — both ways.** The inventory twice
   flags as unverified "whether `graph_content_hash`'s field list also covers the graph-level
   `field_stack`/`principal_field` attrs". Settled by reading both halves:
   `tools/regression_test.py:958-963` builds the digest from `graph.nodes(data=True)` and
   `graph.edges(data=True)` **only**, and its docstring at `:939-943` states *"Graph metadata
   (`g.graph`: economy, event log, opposition states) is also excluded"* — so
   `field_stack`/`principal_field`, written to `G.graph[...]` by
   `world_state.py:801-814`'s `_write_field_stack_graph_attrs`, are **NOT** byte-gated. But
   `to_graph()` also runs `_restamp_field_stack` (`world_state.py:830-876`, called at
   `:780`), which puts `contradiction_fields`/`field_derivatives` back onto **nodes**
   (`:855-867`) and `field_gradients` back onto **edges** (`:869-876`) — so Phases 1-2's
   outputs **ARE** byte-gated, via the round-trip carrier rather than directly. §5's
   live-not-dormant conclusion and §7's "materially better starting position for
   conformance-fixture harvesting" both hold, and the Phase-1 acceptance checklist item can
   now be closed rather than carried.

7. **CONFIRMATION — Phase 0's collapse, the dead defines, and the RESERVED-LINE check.**
   `field_registry` is never wired outside `tests/` (`rg`-verified: only
   `kernel/services.py:43`, `engine/services.py:196`, the two consuming branches, and three
   corroborating comments in `world_state.py:70` / `sentinels/seam/registry.py:1584,1684`),
   so `_discover_field_names` provably yields `["atomization","exploitation"]`;
   `history_window` and `curvature_alpha` are read nowhere in `src/` (`rg`-verified for
   `history_window`: `defines.yaml:404` and `config/defines/consciousness.py:297` only).
   Tick position 20.0 confirmed (`field_derivative.py:47`) against `_DEFAULT_SYSTEMS`'
   position-sorted derivation (`simulation_engine.py:376-378`). "No RESERVED-LINE surface" is
   upheld — the system is gradient/Laplacian/finite-difference calculus over values computed
   upstream, with no doctrine content, no National Question parameter and no outcome
   definition. (The one hardcoded-string special case, `1.0 if f == "exploitation" else 0.0`
   at `:416`, is a tiebreak preference between two mechanical field names, not a theoretical
   commitment.)

**FINAL VERDICT: BLOCKED — on ONE gap, not two, and it is the substrate, not the verb
algebra.** Phase 1 (`field_gradients`) needs `GraphSubstrate` edge-attribute storage on both
read and write — a hash-relevant Constitution III.7 decision, deeper than query slice 2 and
unscheduled on all four named slices (confirmation 4). The second, "entirely unscheduled"
gap the verdict names — no BSL verb writes a graph-level attribute — is **withdrawn**: §3.6
rules graph-scope state to be node state on a `:ceiling 1` carrier, `update-node` is the
verb, and the carrier is reachable under landed Slice 1 without `the` and without
`(domain :graph)` (corrections 1 and 2). Phases 3 and 4 therefore move to **PORTABLE WITH
D-RECORDS** — Phase 3 with the Option-representation D-record its `field_name` axis needs
(correction 3), Phase 4 remaining NOT-A-PACK on its own merits as engine plumbing, now for
that reason alone rather than for a storage blocker it does not have. Phase 2's disposition
is upheld unchanged, with both D-records (edge-type enumeration; the witnessed
double-counting divergence) intact — they remain the real content-modeling risk in this
system.

**INADEQUATE-COVERAGE — a re-read must add:**
(a) `docs/reference/bsl-language.rst` **§3.6** (`:2639-2689`) to the reference list, and a
re-adjudication of the Phase-3 and Phase-4 rows against the carrier ruling — the report reads
§2.3, §2.5, §2.6 and four Rust modules but not the chapter that decides its second-largest
claim;
(b) `tick.rs`'s `subject_type_of`/`run_tick` **subject-population** semantics (`:159-181`,
`:536-538`) — the report reads `tick.rs` for what `run_tick` *does not* do (`loaded.domain`)
without recording what it *does* (iterate `graph.nodes(&subject_type)`), which is what makes
the ceiling-1 carrier fire once per tick;
(c) `scenario.rs`'s `defvocabulary` path (`:389-395`, `:811-850`) plus
`content/scenarios/organization-foundation.bscn:41`, which prove a carrier `NodeType` is
content-declarable on dev today;
(d) the Option gap carried forward onto Phase 3's `field_name` axis;
(e) the §5/§7 byte-gate flag, now closed above in both directions — carrying it as UNVERIFIED
into a port plan would leave a false open question on the acceptance checklist.
