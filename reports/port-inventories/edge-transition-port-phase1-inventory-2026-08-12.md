# EdgeTransitionSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `EdgeTransitionSystem` (`src/babylon/engine/systems/edge_transition/_legacy.py`,
859 lines, System #21, tick position 21.0) evaluates a **16**-entry compound-predicate state machine
(plus one manually-added `ANTAGONISTIC→ANTAGONISTIC` self-persistence pair with no predicate of its
own — `_legacy.py:462` — bringing `_VALID_TRANSITIONS` to 17 total valid `(from,to)` pairs, matching
the source's own FR-010 comment "17 permissible transitions"; verified by direct count: 16
`name=`/`priority=` sites, `_legacy.py:99-444`) over `edge_mode` and fires qualitative-mode
transitions on **edges** — the frozen system's entire
read/write surface, across all three of its declared phases, is fundamentally edge-attribute-shaped
(`edge_mode`, `contradiction_character`, `co_optive_suppressed_fields`, `_dominant_party`), none of
which is declared on the `Relationship` Pydantic model (which `extra="forbid"`s unknown fields) and
none of which survives a `WorldState` graph round-trip. On the **current dev-tree BSL surface**,
edge-attribute reads and writes are a **verified, precisely-named hard blocker**: `evaluator.rs:503`
lists `edges`/`edge-between`/`the` as Slice-2 unserved expression heads (only `nodes`/`neighbors`
are served, `evaluator.rs:527`), `field-of` explicitly cannot produce or consume an `EdgeRef`
(`evaluator.rs:1190-1195`, "no expression form produces one yet"), and `update-edge` — while
grammar/typecheck-recognized (D35/D65) — is **refused at execution on every path**
(`structural_verbs.rs:387-397` and `:709-716`) because `GraphSubstrate` stores an edge as one bare
`f64` strength with **no room for declared fields at all**, a named Constitution-III.7
state-hash-widening decision, not an oversight. Independently, the system is **provably dormant
on every canonical `qa:regression` scenario** (`tools/regression_scenarios.py:2856-2866`, a
declared coverage gap) and, deeper than that, dormant on its own *only* live production write path:
`ElectoralSystem`'s CO_OPTIVE coupling (`electoral.py:494-534`) connects Organization↔Sovereign
nodes, which never carry `contradiction_fields`/`field_derivatives` (restricted to `social_class`
nodes everywhere in the engine, grep-confirmed) — so even a hypothetical live edge would see every
"value" condition default to `0.0` and every "df_dt" condition return `False`, which (verified by
hand-tracing the 16-entry table) makes `concessions_withdrawn` (imperial_rent value `0.0 < 1.0`)
fire **vacuously true** and flip any CO_OPTIVE edge straight back to EXTRACTIVE the very next tick —
a genuine, verified frozen-code oddity to transcribe port-as-is, not repair. The 16 transition
thresholds are also frozen at Python **import time** via a bare `GameDefines()` (`_legacy.py:94`,
never `.load_default()`), so — unlike `co_optive_suppression_rate`/`latent_release_multiplier`,
which ARE live-read from `services.defines` every tick — they are already immune to `defines.yaml`
edits in the frozen system, which favorably matches the `defconst` port convention. Zero libm
transcendentals; zero numeric clamps anywhere in the file; 4 distinct `EventType` emissions.

**Verdict: BLOCKED — the edge-attribute lane (Slice 2: `edges`/`edge-between`/`EdgeRef` field
access) plus a genuine, declared `GraphSubstrate` storage gap for `update-edge`.** Every one of the
system's four computations needs it; none is portable without it.

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/edge_transition/_legacy.py` | 859 | **The target.** `EdgeTransitionSystem` (563-690), the Pydantic predicate/transition models (`PredicateCondition` 39-56, `CompoundPredicate` 59-68, `EdgeModeTransition` 71-84), `_build_transitions` (92-447, the 16-entry table — see the §2 count clarification), `_evaluate_condition`/`_evaluate_predicate` (484-555), `_co_optive_suppression` (693-741), `_handle_co_optive_breakdowns` (744-794), `_check_aspect_reversal` (797-858), `_regime_code` (476-481). Read completely, line by line. Self-contained — no calls into `formulas/` or `domain/`. |
| `src/babylon/engine/systems/edge_transition/__init__.py` | 36 | Spec-059 US5 package split: re-exports `_legacy.py`'s public surface unchanged (byte-equality preserved; the `predicates.py`+`system.py` split it documents was deferred and never landed). |
| `src/babylon/config/defines/consciousness.py` | 582 (whole module); `ContradictionFieldDefines` 272-332, `EdgeTransitionDefines` 334-439 | Coefficient source. This system reads `contradiction_field.co_optive_suppression_rate` (313-318, live per-tick) and `.latent_release_multiplier` (319-324, live per-tick) — NOT `field_min`/`field_max`/`history_window`/`curvature_alpha` (those belong to `ContradictionFieldSystem`). It also reads all 16 `EdgeTransitionDefines` threshold fields (344-439) — but **only once, at Python import time**, via a bare `GameDefines()` (see §2, §4). `default_transition_priority` (327-331) is declared but read nowhere in `src/babylon` (grep-confirmed) — dead. |
| `src/babylon/data/defines.yaml` | `contradiction_field:` 401-408; `edge_transition:` 486-502 | Player-editable coefficient values — the `edge_transition:` block is **never actually consulted at runtime** by this system (see §2 finding). |
| `src/babylon/models/enums/topology.py` | 253 | `EdgeMode` (StrEnum, 130-155: EXTRACTIVE/TRANSACTIONAL/SOLIDARISTIC/ANTAGONISTIC/CO_OPTIVE). `EdgeType.TRANSACTIONAL`/`.ANTAGONISTIC`/`.SOLIDARISTIC` (114-117) are a **distinct, same-named enum** (Organization↔Community edges, Feature 032/039) — the module's own docstring (edge_transition `_legacy.py:1-11`, "R-002") flags EdgeMode vs EdgeType as a deliberate distinction. |
| `src/babylon/models/enums/consciousness.py` | 93 | `ContradictionCharacter` (StrEnum, 49-65: ANTAGONISTIC/NON_ANTAGONISTIC). |
| `src/babylon/models/enums/events.py` | 234 | `EventType.EDGE_MODE_TRANSITION` (100), `.CO_OPTIVE_BREAKDOWN` (105), `.LATENT_CONTRADICTION_RELEASE` (106), `.ASPECT_REVERSAL` (107) — the 4 emissions. |
| `src/babylon/kernel/event_bus.py` | 288 | `Event` (frozen dataclass, 33-50: `type: str`, `tick: int`, `payload: dict[str, Any]`), `.publish(...)` — every `services.event_bus.publish(Event(...))` call site. |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase._get_persistent_data` (199-202) — the only helper this system uses. `_write_clamped` (162-192) exists but is **never called** (no clamp anywhere in this system, see §4). |
| `src/babylon/kernel/graph_protocol.py` | 494 | `GraphProtocol.query_edges` (278-298, no `edge_type` filter passed — a full scan of every edge every tick), `.get_node` (77-86), `.update_edge` (152-170), `.get_graph_attr` (350-363). Never calls `query_nodes`/`add_edge`/`remove_edge`/`set_graph_attr`. |
| `src/babylon/kernel/system_protocol.py` | 41 | `ContextType` alias. |
| `src/babylon/kernel/tick_partition.py` | 30 | `TickPartition.CONSEQUENCE` — `_legacy.py:573`. |
| `src/babylon/kernel/services.py` | 88 | `ServicesProtocol.event_bus: EventBus` (typed precisely), `.defines: Any` (38, structurally `GameDefines`). |
| `src/babylon/topology/graph.py` | 1033 | Concrete `BabylonGraph.update_edge` (690-, plain dict merge — no coercion/quantization, same runtime-storage caveat every other ported system carries), `.get_node` (651-), `.get_graph_attr`/`.set_graph_attr` (892-898, a flat `dict[str, Any]` keyed store unrelated to node/edge storage). |
| `src/babylon/topology/adapters/query_mixin.py` | 146 | `QueryMixin.query_edges` (70-) — the concrete iteration `BabylonGraph` inherits; no edge-type index, a full attribute-dict scan. |
| `src/babylon/models/entities/relationship.py` | 187 | `Relationship` — the canonical edge entity model. `model_config = ConfigDict(extra="forbid", frozen=True, ...)` (51-55). Declares `value_flow`/`tension`/`subsidy_cap`/`solidarity_strength`/`influence_level`/`support_type`/`control_level`/`legal_status` — **none of `edge_mode`/`contradiction_character`/`co_optive_suppressed_fields`/`_dominant_party`**. Load-bearing for §3 and §5. |
| `src/babylon/models/world_state.py` | 1161 | `_reconstruct_relationships` (357-393) — explicit docstring: "any other edge attribute a system writes is dropped on reconstruction." `to_graph()` (658-) writes "all Relationship fields as attributes" only. `_restamp_field_stack` (831-) restores node-side `contradiction_fields`/`field_derivatives` and edge-side `field_gradients` after a round-trip — but has **no equivalent carrier** for any of this system's own edge attributes. Confirms the round-trip-drop finding in §3/§5. |
| `src/babylon/engine/systems/electoral.py` | (relevant: 494-534) | `ElectoralSystem._accrue_commit_coupling` — the **only** production write site for `edge_mode` besides this system itself: sets `edge_mode=EdgeMode.CO_OPTIVE.value` on an org→sovereign `TRANSACTIONAL` edge, gated on a non-empty `committed` list AND a resolvable `_defended_apex` sovereign. Position 17.45, three positions before this system in the same tick. |
| `src/babylon/engine/systems/doctrine.py` | (relevant: 197) | The **only** production reader of `edge_mode` besides this system: `if str(edge.attributes.get("edge_mode", "")) == EdgeMode.CO_OPTIVE.value`. Position 14.7 — reads the **prior tick's** value (runs before this system in tick order). |
| `src/babylon/engine/systems/contradiction_field.py` | 259 | `ContradictionFieldSystem` @19.0, TWO positions before this system. `_OPPOSITION_FIELD_NAMES = ("exploitation", "atomization")` (47) and `_step_from_oppositions` (153-202) — the **production-live** path (registry is always `None`, see §5) that determines exactly which `contradiction_fields` keys this system can ever read for real. `graph.query_nodes(node_type=NodeType.SOCIAL_CLASS)` (110, 184) — restricts field writes to social_class nodes only. |
| `src/babylon/engine/systems/field_derivative.py` | 457 | `FieldDerivativeSystem` @20.0, ONE position before this system (its immediate predecessor). `graph.update_node(node_id, field_derivatives=field_derivatives)` (237) and `_discover_field_names` (106-124, also `NodeType.SOCIAL_CLASS`-restricted, 120) — confirms `field_derivatives` inherits the same exploitation/atomization-only key space. |
| `src/babylon/engine/systems/contradiction.py` | 1127 (relevant: 103, 1039-1089) | `ContradictionSystem` @18.0, three positions before. `DIALECTICAL_REGIME_ATTR = "dialectical_regime"` (103); `_classify_regime` (1039-1089) writes the graph-level `dialectical_regime` attr (`graph.set_graph_attr` at 1086-1089) this system's dead `_regime_code` reads. |
| `src/babylon/engine/services.py` | 459 | `ServiceContainer.create()`'s `resolved_defines = defines if defines is not None else GameDefines()` (377) — bare constructor default, same pattern as `_build_transitions()`'s own bare `GameDefines()`. |
| `src/babylon/engine/simulation_engine.py` | 611 | `_SYSTEM_CLASSES` (328-363) — confirms tick order: `ContradictionSystem@18.0 → ContradictionFieldSystem@19.0 → FieldDerivativeSystem@20.0 → CollapseTransitionSystem@20.5 → EdgeTransitionSystem@21.0 → WealthDistributionSystem@21.5 → EpistemicHorizonSystem@22.0`. Position ClassVars confirmed by direct grep of each system file (§5). Production entrypoint `effective_defines = defines if defines is not None else GameDefines.load_default()` (560) — the ONLY place `defines.yaml` is actually consulted for a real game run, and `_build_transitions()`'s module-level table never sees it. |
| `src/babylon/config/defines/_assembler.py` | 411 | `GameDefines.load_default()` (393-408) — reads `defines.yaml` if present; bare `GameDefines()` uses schema `Field` defaults only. |
| `src/babylon/models/entities/organization.py`, `src/babylon/models/entities/sovereign.py` | — | Grep-confirmed: **no `wealth` field declared on either** — the node types the one live `edge_mode` write path (`electoral.py`) actually connects. |
| `tools/regression_scenarios.py` | (relevant: 2856-2866) | `COVERAGE_GAPS_DATA` entry for `EdgeTransitionSystem` — the canonical, already-declared dormancy finding (§5). |
| `tools/regression_test.py` | (relevant: 924-964) | `graph_content_hash` — hashes every node/edge attribute of the `WorldState→graph` projection; would catch drift on this system's outputs IF any canonical scenario ever seeded them (it does not, see §5, §7). |
| `rust/crates/babylon-bsl/src/evaluator.rs` | — | `UNSERVED_EXPRESSION_HEADS` (486-512): `edges`/`edge-between`/`the` named Slice 2. `SERVED_QUERY_HEADS` (514-527): only `["nodes", "neighbors"]`. `eval_field_of` doc (1190-1195): "an `EdgeRef` referent is unreachable today." **Verified myself against dev tip** — the central blocker citation for §6. |
| `rust/crates/babylon-bsl/src/structural_verbs.rs` | — | Module doc (14-25) + both match arms (387-397, 709-716): `update-edge`/`update-hyperedge` are grammar-recognized (D35/D65) but refused at EVERY execution path with a named reason: `GraphSubstrate` keys an edge to one bare `f64` strength, "gives a hyperedge no attributes at all," and widening either is a declared Constitution-III.7 state-hash decision, not a silent gap. **Verified myself against dev tip.** |

## 2. COMPUTATION CATALOG (execution order, `_legacy.py:580-690`)

**Transition-count clarification (verified by direct enumeration, corrects an initial miscount):**
`_build_transitions()` constructs exactly **16** `EdgeModeTransition` objects (16 `name=`/`priority=`
sites, `_legacy.py:99-444`), each a predicate-gated `(from_mode, to_mode)` arc with priority and
description. `_VALID_TRANSITIONS` (458-460) is built from those 16 pairs, then ONE more pair is
added directly — `(ANTAGONISTIC, ANTAGONISTIC)`, "also valid (persistence)" (461-462) — with **no**
`EdgeModeTransition` object, no predicate, no priority, and no `_TRANSITION_MAP` entry (that map is
built only from `_TRANSITIONS`, the 16 real objects, at 453-455). This reconciles the source's own
FR-010 comment ("17 permissible transitions," line 88): it counts valid `(from,to)` **pairs** (17),
not predicate-bearing transition **objects** (16). The self-loop pair matters only for
`_VALID_TRANSITIONS` set-membership checks (the L1 closure law tests below); Phase 2b's firing loop
can never select it (it has no predicate to fire, and line 657's `if new_mode != current_mode` guard
would suppress writing it even if it somehow appeared as "fired"). Below, "16-entry table" refers to
the real transitions; "17 valid pairs" refers to `_VALID_TRANSITIONS`.

### Phase 0 — Setup (`step()`, `_legacy.py:594-603`)
- **(a)** Read the tick number; fetch (or lazily create) the `persistent_data["latent_contradictions"]` accumulator dict.
- **(b)** `tick: int = context.tick` (597); `latent = persistent_data.setdefault("latent_contradictions", {})` (601-603).
- **(c) Reads:** `context.tick`, `context.persistent_data`.
- **(d) Writes:** `persistent_data["latent_contradictions"]` — created as `{}` if absent (this is the **one unconditional write** `tools/regression_scenarios.py:2860-2863` names as "an empty-dict bookkeeping stamp, not material logic" on every canonical scenario).
- **(e) Defines:** none.
- **(f) Events:** none.

### Phase 1 — CO-OPTIVE suppression (`_co_optive_suppression`, `_legacy.py:693-741`)
- **(a)** For every edge currently in CO_OPTIVE mode that names a list of "suppressed" contradiction fields, accumulate that field's suppressed derivative (df/dt × a rate) into a cross-tick, node-and-field-keyed latent-contradiction ledger — a form of dialectical repression: the tension doesn't disappear, it's stored up.
- **(b)** `suppressed = df_dt * suppression_rate` (740); `node_latent[field_name] = node_latent.get(field_name, 0.0) + suppressed` (741) — only when `df_dt is not None and df_dt > 0` (739, EC-guard: a falling/undefined derivative contributes nothing, never subtracts).
- **(c) Reads:** ALL edges (`graph.query_edges()`, 710, no type filter), `edge_attrs.get("edge_mode")` (712), `edge_attrs.get("co_optive_suppressed_fields")` (722, a `list[str]`), the CO_OPTIVE edge's SOURCE node's `field_derivatives[field_name]["df_dt"]` (732-738).
- **(d) Writes:** `latent[edge.source_id][field_name]` — **not a graph write**; a plain Python dict mutation of the `persistent_data` accumulator passed by reference from `step()`. No `graph.update_node`/`update_edge` call in this function at all.
- **(e) Defines:** `contradiction_field.co_optive_suppression_rate` (708, `[0.0, 1.0]`, default 1.0) — read **live** from `services.defines` every tick (contrast with Phase 2's frozen table, §4).
- **(f) Events:** none.

### Phase 2a — `contradiction_character` default stamp (`step()`, `_legacy.py:625-632`)
- **(a)** Every edge that carries `edge_mode` but has never been touched by this system before gets `contradiction_character` initialized to NON_ANTAGONISTIC.
- **(b)** `if "contradiction_character" not in edge_attrs: graph.update_edge(..., contradiction_character=ContradictionCharacter.NON_ANTAGONISTIC)` — a key-**absence** check (Python `in`/`not in` on the attribute dict), not a value comparison.
- **(c) Reads:** `edge_attrs` (the full edge attribute dict, for the `in` check).
- **(d) Writes:** `EDGE.contradiction_character` (conditionally).
- **(e) Defines:** none.
- **(f) Events:** none.

### Phase 2b — Predicate evaluation + mode transition (`step()`'s main loop + `_evaluate_condition`/`_evaluate_predicate`, `_legacy.py:613-684`, `484-555`)
- **(a)** For every edge carrying a parseable `edge_mode`, look up the transitions whose `from_mode` matches, evaluate each one's compound (AND-of-conditions) predicate against the source/target node's contradiction-field values and derivatives, and — if one or more predicates fire — transition to the highest-priority fired transition's `to_mode`, emitting an event.
- **(b)** Per-condition: `value = fields.get(condition.field, 0.0)` for `metric="value"` (507-508, **silent 0.0 default on a missing key**, not an error); for `metric` in `{df_dt, d2f_dt2, laplacian}`, `raw = field_deriv.get(condition.metric); if raw is None: return False` (509-514, EC-001, "undefined derivative cannot satisfy predicate" — a **different** missing-data policy than the value branch: silent-zero vs. hard-false); for `metric="regime"`, reads the tick's `dialectical_regime` ordinal (516-519, dead — no shipped transition uses it, see below). Comparison: `gt`/`lt`/`gte`/`lte` against `condition.threshold` (524-532). Compound: `all(...)` over conditions (552-555, AND-only — no OR/NOT in the predicate language). Selection: `best = max(fired, key=lambda t: t.priority)` (654) — Python's `max()` first-element-wins on a tie; **never actually exercised**, because every `from_mode` group's priorities are pairwise distinct in the shipped table (verified by hand-checking all 16 entries' priorities per group — no two transitions sharing a `from_mode` share a `priority`).
- **(c) Reads:** ALL edges (`graph.query_edges()`, 613); `edge_attrs.get("edge_mode")` (615); `graph.get_node(edge.source_id)`/`.get_node(edge.target_id)` (635-636); `source_attrs`/`target_attrs`' `contradiction_fields` (dict, 507) and `field_derivatives` (nested dict, 510); `graph.get_graph_attr("dialectical_regime", None)` (478, via `_regime_code`, called unconditionally at 609 regardless of whether any transition needs it).
- **(d) Writes:** `EDGE.edge_mode` (only when `new_mode != current_mode`, 657-663).
- **(e) Defines:** all 16 `EdgeTransitionDefines` threshold fields (§1, §4 — frozen at import time, defines.yaml never consulted).
- **(f) Events:** `EventType.EDGE_MODE_TRANSITION` (665-678, payload: `source_id`, `target_id`, `from_mode`, `to_mode`, `predicate` name, `description`).
- **VERIFIED DORMANCY / ODDITY (production-specific, hand-traced):** because `contradiction_fields`/`field_derivatives` are written ONLY on `social_class` nodes (`contradiction_field.py:110`, `field_derivative.py:120`) while the system's one live write path (`electoral.py:494-534`) only ever sets `edge_mode` on an Organization→Sovereign edge, **every "value" condition on such an edge defaults to `0.0`** and **every "df_dt" condition returns `False`** (the EC-001 branch). Hand-tracing the 4 CO_OPTIVE-departure transitions against this: `co_optive_breakdown` (needs exploitation df_dt, undefined→False), `co_optation_normalizes` (needs exploitation value AND df_dt, df_dt undefined→False), `co_optation_recognized` (needs exploitation value on both sides >5.0, defaults 0.0→False), `concessions_withdrawn` (needs imperial_rent value <1.0 on target — defaults to `0.0`, and `0.0 < 1.0` is **True**) — so `concessions_withdrawn` is the sole fired transition, **vacuously true by missing-field default**, flipping the edge CO_OPTIVE→EXTRACTIVE the very next tick this system runs. This is a genuine frozen-code behavior to transcribe verbatim (port-as-is law), not a bug to fix.

### Phase 2c — Aspect reversal detection (`_check_aspect_reversal`, `_legacy.py:797-858`, called once per edge inside the same loop at line 687)
- **(a)** Track which endpoint of a directed edge is "dominant" by material wealth; when the dominant party switches, emit an event.
- **(b)** `source_wealth = float(source_attrs.get("wealth", 0.0)); target_wealth = float(target_attrs.get("wealth", 0.0))` (822-823); three-way branch: source wealth strictly greater → source dominant; target strictly greater → target dominant; **equal → no change** (`current_dominant = previous_dominant`, 830 — a "sticky" tie, not a fixed tiebreak side). Write only `if current_dominant is not None` (833-839, so a first-ever tie with no prior `_dominant_party` writes nothing at all). Event only when `previous_dominant`, `current_dominant` are both non-None AND differ (842-846).
- **(c) Reads:** `edge_attrs.get("_dominant_party")` (819); `source_attrs.get("wealth", 0.0)`/`target_attrs.get("wealth", 0.0)` (822-823).
- **(d) Writes:** `EDGE._dominant_party` (conditionally, 833-839).
- **(e) Defines:** none.
- **(f) Events:** `EventType.ASPECT_REVERSAL` (847-858, payload: `source_id`, `target_id`, `previous_dominant`, `new_dominant`).
- **Note:** this call is nested INSIDE the same `if current_mode_str is None: continue` guard as Phase 2b (line 616-617's `continue` skips line 687 too) — so aspect-reversal detection **only ever runs on edges that already carry `edge_mode`**, even though its own logic has nothing to do with edge mode. On the one live write path (`electoral.py`, Organization↔Sovereign, no `wealth` field on either model — grep-confirmed), both wealths default to `0.0`, are always equal, `previous_dominant` starts `None` and stays `None` forever — `ASPECT_REVERSAL` can never fire in production on that edge either.

### Phase 3 — CO-OPTIVE breakdown handling (`_handle_co_optive_breakdowns`, `_legacy.py:744-794`, driven by the `co_optive_breakdowns` list Phase 2b appended to at lines 681-684 whenever a CO_OPTIVE edge transitioned away)
- **(a)** When an edge leaves CO_OPTIVE mode this tick, release its accumulated latent contradiction as an amplified spike, clear the accumulator, and emit two events.
- **(b)** `node_latent = latent.pop(source_id, {})` (766, unconditional removal — pop-or-default, never raises); skip if empty (767-768); `released_fields = {f: v * multiplier for f, v in node_latent.items()}` (791) for the second event's payload — the multiply happens **only in the event payload construction**, never written back to any state.
- **(c) Reads:** `latent[source_id]` (the same `persistent_data` dict Phase 1 wrote); `co_optive_breakdowns` (the in-`step()`-scope local list, not persistent_data — Phase 2b→Phase 3 communication is a plain Python variable, tighter than even a `persistent_data` round trip).
- **(d) Writes:** `latent` (node removed via `.pop`) — again not a graph write.
- **(e) Defines:** `contradiction_field.latent_release_multiplier` (763, `[1.0, 5.0]`, default 1.5) — live per-tick.
- **(f) Events:** `EventType.CO_OPTIVE_BREAKDOWN` (771-782, payload: `source_id`, `target_id`, `latent_released` dict, `multiplier`); `EventType.LATENT_CONTRADICTION_RELEASE` (785-794, payload: `node_id`, `released_fields` dict).

**Dead code, verified structurally unreachable given the shipped table (not a Phase, listed for completeness):** `_evaluate_condition`'s `d2f_dt2`/`laplacian`/`regime` branches (509-521) and the `_REGIME_CODES` table (473) are never invoked by any of the 16 shipped transitions — grep-confirmed only `"value"` and `"df_dt"` appear as `metric=` literals across all of `_build_transitions()` (§4). `_regime_code(graph)` (609) still runs every tick regardless (a pure-waste read of the `dialectical_regime` graph attr with zero consumers in the live table). `CompoundPredicate`/`PredicateCondition` are constructed nowhere outside `_build_transitions()` in `src/` (grep-confirmed) — the generic multi-metric interpreter exists, but production only ever exercises two of its five metric branches.

**Events emitted by the whole system: 4 distinct `EventType`s** (`EDGE_MODE_TRANSITION`, `CO_OPTIVE_BREAKDOWN`, `LATENT_CONTRADICTION_RELEASE`, `ASPECT_REVERSAL`) — grep-confirmed exhaustive.

## 3. TYPE INVENTORY

Runtime storage note (same as every other ported system): `BabylonGraph.update_edge`/`.update_node`
(`topology/graph.py:690-`, `:651-`) are plain dict merges with no coercion or quantization — all
in-tick arithmetic is raw Python `float`/`bool`/`str`, never grid-snapped mid-tick.

**Structural note load-bearing for this system specifically (not present in Territory's inventory):**
`Relationship` (`models/entities/relationship.py`) is the canonical edge Pydantic model and it
declares `model_config = ConfigDict(extra="forbid", ...)` (51-55) — yet **none of the four edge
attributes below are Relationship fields**. They exist purely as `graph.attributes` dict keys,
typed only by the enum classes constructed at each read/write call site (or, for the two untyped
ones, not typed at all anywhere). This is categorically different from Territory (whose target
entity model declares every field it touches).

| Attribute | Node/Edge type | Python model type | Domain | Category |
|---|---|---|---|---|
| `edge_mode` | EDGE (any `edge_type`) | `EdgeMode` (StrEnum, 5 members) — **not a `Relationship` field**; parsed defensively via `EdgeMode(str)` with a `try/except ValueError: continue` fallback | `{EXTRACTIVE, TRANSACTIONAL, SOLIDARISTIC, ANTAGONISTIC, CO_OPTIVE}` | **Enum discriminant, undeclared on any Pydantic model** |
| `contradiction_character` | EDGE | `ContradictionCharacter` (StrEnum, 2 members) — **not a `Relationship` field** | `{ANTAGONISTIC, NON_ANTAGONISTIC}` | **Enum discriminant, undeclared** |
| `co_optive_suppressed_fields` | EDGE | `list[str]` — **not a `Relationship` field; never written anywhere in `src/`** (grep-confirmed — only read, by this system) | open (arbitrary field-name strings) | **Untyped list-of-string, undeclared, unseeded anywhere** |
| `_dominant_party` | EDGE | `str \| None` (a node ID) — **not a `Relationship` field** | any valid node ID, or `None` | **Untyped optional string, undeclared** |
| `contradiction_fields` | NODE (social_class only, in production) | `dict[str, float]` — **not a `SocialClass` field** (confirmed via `EXTRA_STAMPABLE_ATTRIBUTES`-style graph-only attribute, same pattern as Territory's non-model attrs) | keys observed in production: `{"exploitation", "atomization"}` only (§5) | **Nested map, no BSL field type** |
| `field_derivatives` | NODE (social_class only) | `dict[str, dict[str, float \| None]]` — doubly-nested | sub-keys: `{"laplacian", "df_dt", "d2f_dt2"}`, values `float \| None` | **Doubly-nested map, no BSL field type** |
| `wealth` | NODE (any type EdgeTransitionSystem's endpoints happen to be) | `Currency` on `SocialClass` (`social_class.py:57`, `Annotated[float, ge=0.0]`) — **absent entirely** on `Organization`/`Sovereign` (grep-confirmed) | `[0, ∞)` where declared; implicit `0.0` default elsewhere | unbounded real, **model presence is node-type-dependent** |
| `dialectical_regime` | **GRAPH-LEVEL** attribute (neither node nor edge) | `dict[str, Any]` shaped `{"regime": str, "opposition": str, "rate": float}`, written by `graph.set_graph_attr` | `regime ∈ {"reproduction", "crisis", "sublation"}` (exhaustive, confirmed against `domain/dialectics/core/regime.py:41-61`'s return-type docstring) | **A third storage location** (not node, not edge) — dead-code consumer only (§2) |
| `priority` (`EdgeModeTransition` field) | host-side table data, never on the graph | `int`, `Field(default=0)` | 16 shipped literals (`_legacy.py:121-444`): `{3,5,5,5,5,5,6,6,7,7,8,8,10,10,10,10}` — pairwise distinct within every `from_mode` group (no domain constraint, hardcoded per-transition) | compile-time table data |
| `threshold` (`PredicateCondition` field) | host-side table data | `float`, unconstrained on the Pydantic field itself | bounded only by the sourcing `EdgeTransitionDefines` field (`[0.0, 10.0]`) | compile-time table data once resolved |
| 16 `edge_transition.*_threshold` defines | — | `float` | `[0.0, 10.0]` each, `consciousness.py:344-439` | unit coefficients, **frozen at import time, never live** |
| `co_optive_suppression_rate` (define) | — | `float` | `[0.0, 1.0]`, default 1.0 | unit-interval coefficient, **live per-tick** |
| `latent_release_multiplier` (define) | — | `float` | `[1.0, 5.0]`, default 1.5 | **amplification-only coefficient** (`ge=1.0` — schema-enforced never-shrinks), **live per-tick** |
| `default_transition_priority` (define) | — | `int` | `≥ 0`, default 0 | **dead — read nowhere** (`consciousness.py:327-331`) |
| `latent[node_id][field_name]` (persistent_data accumulator) | — | `dict[str, dict[str, float]]`, cross-tick, host-side only | unbounded above (monotone non-decreasing only — **zero clamp anywhere**, unlike every clamped field in Territory/Metabolism) | **dynamically-keyed nested accumulator, no BSL storage location of any kind** (not node, not edge, not graph attr — pure host-side `persistent_data`) |

**Round-trip hazard (verified, `models/world_state.py:357-393`, `658-`, `831-`):** `_reconstruct_relationships`
explicitly documents "any other edge attribute a system writes is dropped on reconstruction," and its
field list omits all four of this system's edge attributes. `to_graph()` writes "all Relationship
fields as attributes" only. `_restamp_field_stack` (831-) restores node-side
`contradiction_fields`/`field_derivatives` and edge-side `field_gradients` after a round-trip via a
dedicated carrier (`field_stack`) — but there is **no equivalent carrier for `edge_mode` /
`contradiction_character` / `co_optive_suppressed_fields` / `_dominant_party`**. A `WorldState`
save/load cycle silently deletes this system's entire edge-side state while preserving the node-side
state it reads. This is a live production hazard independent of any port question.

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `float`), zero libm transcendentals — grep-confirmed zero
`exp`/`log`/`sigmoid`/`pow`/`math.` calls anywhere in the file. Zero `int()` casts (no Real→Int
demotions). **Zero numeric clamps anywhere in this system** — grep-confirmed the only `min`/`max`
call in the file is `max(fired, key=lambda t: t.priority)` (654, a selection, not a bound). This
is a structural difference from Territory (`heat` clamped two different ways) and from Metabolism —
`latent[node][field]`'s accumulator, in particular, is unbounded above by construction (§3). Shapes,
in execution order:

1. **Threshold comparison (×4 operator shapes):** `value > threshold` / `< ` / `>=` / `<=`
   (`_legacy.py:524-531`) — plain float comparisons, no clamp, no epsilon tolerance.
2. **Multiplicative suppression:** `df_dt * suppression_rate` (740) — one multiply, guarded by
   `df_dt is not None and df_dt > 0` (739).
3. **Additive accumulation, unbounded:** `node_latent.get(field_name, 0.0) + suppressed` (741) —
   monotone non-decreasing (Law L3, `tests/unit/engine/laws/test_law_edge_transition.py:249-`), but
   **never capped** — contrast Territory's `_write_clamped [0,1]`.
4. **Amplifying multiply:** `v * multiplier` (791, dict comprehension) — `multiplier ∈ [1.0, 5.0]`
   schema-enforced `ge=1.0`, so `released ≥ accumulated` always (Law L4).
5. **Selection max with an unexercised tiebreak:** `max(fired, key=lambda t: t.priority)` (654) —
   Python's `max()` is first-max-wins on a tie; **verified never exercised** — every `from_mode`
   group's 16 priorities are pairwise distinct hardcoded literals (not defines, so not moddable
   into collision either). Structurally favorable match to the landed `select-max`'s "language-level
   tiebreak" (per the current BSL surface) — but untested by this system's own shipped data.
6. **Wealth comparison, no arithmetic:** `source_wealth > target_wealth` / `<` / tie→sticky-previous
   (822-830) — a three-way branch, not a subtraction-then-compare; the "gap" itself is never computed
   as a value, only the ordering.
7. **Bare literal `0.0`** appears as: (a) an explicit threshold in 4 of the 16 transitions (`df_dt`
   compared against `0.0`, lines 116, 295, 352, 378); (b) the default-value literal in every `.get(key, 0.0)` call
   (multiple sites); (c) inside `_REGIME_CODES = {"reproduction": 0.0, "crisis": 1.0, "sublation":
   2.0}` (473, dead code, §2). Per the established BSL parser constraint ("no bare non-integer
   literal" — Territory's inventory, §4 finding #2), each of these needs the `c`-suffixed or
   Real-zero-promotion idiom already precedented in the reference packs — a mechanical, not
   structural, concern once the surrounding edge-lane blocker (§6) is solved.
8. **The 16-transition table's thresholds are frozen at Python IMPORT time, not read live**
   (`_build_transitions()`, `_legacy.py:92-96`: `et = GameDefines().edge_transition` — a **bare**
   constructor, never `GameDefines.load_default()`; `_TRANSITIONS: list[EdgeModeTransition] =
   _build_transitions()` at module scope, `450`). **Contrast within the same file:**
   `_co_optive_suppression` (708) and `_handle_co_optive_breakdowns` (763) read
   `services.defines.contradiction_field.*` **live, every tick** — so editing `defines.yaml`'s
   `edge_transition:` block (486-502) has **zero effect** on a real game run, while editing its
   `contradiction_field:` block (401-408) does. This asymmetry is independently documented (without
   the "modding contract" framing) in the test file's own docstring caveat
   (`tests/unit/engine/laws/test_law_edge_transition.py:47-56`). Verified: `GameDefines.load_default()`
   (`_assembler.py:393-408`) is what actually consults `defines.yaml`; the production tick entrypoint
   (`simulation_engine.py:560`) calls it; `_build_transitions()` never does, at any call site, ever
   (it runs exactly once, at import). **This is favorable for a port**: the 16 thresholds behave
   exactly like compile-time `defconst`s already in the frozen system — the port doesn't lose any
   moddability by baking them in, because the frozen system never offered any.

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 21.0** (`_legacy.py:574`), confirmed against `_SYSTEM_CLASSES`
  (`simulation_engine.py:328-363`) and each neighboring System's own `position` ClassVar
  (grep-verified per-file): `ContradictionSystem@18.0 → ContradictionFieldSystem@19.0 →
  FieldDerivativeSystem@20.0 → CollapseTransitionSystem@20.5 → EdgeTransitionSystem@21.0 →
  WealthDistributionSystem@21.5 → EpistemicHorizonSystem@22.0`. This system is second-to-last in
  the Consequences phase; only `WealthDistributionSystem`/`EpistemicHorizonSystem` run after it
  this tick.
- **Reads from same-tick prior systems:**
  - `contradiction_fields` — written by `ContradictionFieldSystem@19.0`, **two positions earlier**
    the same tick (`contradiction_field.py:138` registry path, `:195` live `_step_from_oppositions`
    path). **In production the registry is always `None`** (`engine/services.py:196`, default;
    grep-confirmed zero non-test call sites pass `field_registry=`) so the LIVE path is exclusively
    `_step_from_oppositions`, which populates **exactly two keys**: `"exploitation"` (mean fresh
    edge `tension`, itself written by `ContradictionSystem@18.0` one position earlier still) and
    `"atomization"` (a graph-level opposition gap). The shipped 16-transition table also references
    `"immiseration"` and `"imperial_rent"` — **neither key is ever populated in production** (§2's
    verified-dormancy finding traces directly from this fact).
  - `field_derivatives` — written by `FieldDerivativeSystem@20.0`, its **immediate predecessor**
    (`field_derivative.py:237`), restricted to the same `contradiction_fields`-key space per node
    (`_discover_field_names`, 106-124) — so `field_derivatives` inherits the exploitation/atomization
    -only restriction transitively.
  - `dialectical_regime` (graph-level attr) — written by `ContradictionSystem@18.0`
    (`contradiction.py:1086-1089`, inside `_classify_regime`, itself gated on a `capital_labor` or
    principal opposition state existing — `contradiction.py:1066-1070`, can leave the attr entirely
    absent some ticks). Read by `_regime_code` (`_legacy.py:476-481`) — but consumed by zero live
    transitions (§2 dead code finding).
  - `wealth` — written by many prior-tick systems across the whole tick (Vitality@1.0,
    Production@3.0, ImperialRent@9.0, Decomposition@11.0, Struggle@16.0, MarketScissors@17.8 being
    the last writer before this system's position, grep-verified across `engine/systems/*.py`) —
    but only meaningfully present on `SocialClass` nodes (`Currency`-typed,
    `social_class.py:57`); `Organization`/`Sovereign` never declare it (grep-confirmed), so on the
    one live edge_mode-bearing edge type this system's aspect-reversal check reads a permanent
    `0.0`/`0.0` tie (§2c).
  - `edge_mode` (cross-tick, own prior write) — the ONLY other production writer besides this
    system itself is `ElectoralSystem@17.45`'s `_accrue_commit_coupling` (`electoral.py:494-534`),
    three positions earlier the same tick, gated on a non-empty `committed` orgs list AND a
    resolvable `_defended_apex` sovereign.
- **Writes consumed downstream:**
  - `edge_mode` — read by exactly ONE other System in `src/babylon/engine/systems/`: `DoctrineSystem@14.7`
    (`doctrine.py:197`, grep-confirmed the only other hit), which — because Doctrine runs at 14.7
    and this system runs at 21.0 — reads **last tick's** value, a cross-tick channel, not
    same-tick.
  - `contradiction_character`, `_dominant_party` — grep-confirmed read by **no other System**.
    Terminal/observational outputs, same category as Territory's `heat`/`rent_level`.
  - `co_optive_suppressed_fields` — grep-confirmed read by **no code anywhere**, not even written
    anywhere (§3) — a pure dead attribute name that exists only as a lookup key.
  - `latent_contradictions` (persistent_data) — read only by this system itself, across ticks
    (Phase 1 writes, Phase 3 reads/pops). No other System touches `persistent_data["latent_contradictions"]`
    (grep-confirmed).
- **Context/service usage with no BSL equivalent:** none beyond `services.defines` (ordinary
  coefficient access) and `services.event_bus.publish` (ordinary event emission) — this system uses
  no `TickContext`-specific override machinery analogous to Territory's `displacement_mode`.
- **DORMANCY — declared and independently verified.** `tools/regression_scenarios.py:2856-2866`
  (`COVERAGE_GAPS_DATA`, `"system": "EdgeTransitionSystem"`): *"no edge in any of the five scenarios
  carries an edge_mode attribute (Relationship does not declare that field, and no scenario factory
  sets it); the 17-transition predicate table never evaluates a real transition. The one
  unconditional write (persistent_data['latent_contradictions'] = {}, from _co_optive_suppression
  finding zero CO_OPTIVE edges) is an empty-dict bookkeeping stamp, not material logic."* Because
  `edge_mode`-absent edges hit the `continue` at line 617 immediately, **Phase 2b and Phase 2c both
  never execute a single loop body** on any canonical scenario; Phase 1 and Phase 3 iterate real
  edges but find zero CO_OPTIVE ones every time (Phase 1's own `if mode != EdgeMode.CO_OPTIVE:
  continue` at line 719 short-circuits every edge). **This inventory goes one layer deeper than the
  declared gap**: even setting the canonical-scenario absence aside, the system's *only* production
  write path (`ElectoralSystem`) connects node types (`Organization`↔`Sovereign`) that never carry
  `contradiction_fields`/`field_derivatives` at all, so a hypothetical scenario that DID seed
  `committed` orgs + a resolvable apex sovereign would still see the vacuous
  `concessions_withdrawn`-always-fires oddity documented in §2, not a materially-driven transition.
  A conformance fixture for this system must be entirely hand-built (as `test_field_topology_integration.py`
  already is) — nothing in the canonical estate, and no realistic extension of it along the
  ElectoralSystem path, exercises the exploitation/immiseration/imperial_rent-driven transitions as
  designed.

## 6. BLOCKER ASSESSMENT

Adjudicated against the CURRENT BSL surface — anchors verified by this agent against dev tip
(`rust/crates/babylon-bsl/src/evaluator.rs`, `structural_verbs.rs`), not merely cited from the
prompt.

| Computation | Verdict | Detail |
|---|---|---|
| Phase 0 setup (persistent_data init, `_legacy.py:594-603`) | **PORTABLE WITH D-RECORD** | `latent_contradictions` is a dynamically-keyed, cross-tick, non-graph accumulator (`dict[str, dict[str, float]]`) — no BSL construct (node field, edge field, graph attr, or otherwise) represents host-side persistent_data of this shape. Needs its own content-modeling decision before any of Phases 1/3 can even be discussed; flagged separately from the edge-lane blocker below because it would block those two phases even if edges were fully solved. |
| Phase 1 CO-OPTIVE suppression (`_co_optive_suppression`, `_legacy.py:693-741`) | **BLOCKED — edge-attribute lane (Slice 2) + no non-graph accumulator construct + no list-field type** | Needs: (a) iterate ALL edges and read `edge_mode`/`co_optive_suppressed_fields` off each — `evaluator.rs:503-527` names `edges` Slice 2, unserved, only `nodes`/`neighbors` served; (b) `co_optive_suppressed_fields` is `list[str]` — `deffield`'s vocabulary (int/bool/currency/probability/intensity/coefficient/enum) has no list-of-string type at all, a gap distinct from and deeper than the enum gap; (c) accumulate into the Phase-0 host-side dict — same blocker as above. |
| Phase 2a `contradiction_character` default stamp (`step()`, `_legacy.py:625-632`) | **BLOCKED — edge-attribute write (Slice 2) + key-absence semantics undefined** | `update-edge` is grammar/typecheck-recognized (D35/D65) but refused at EVERY execution path (`structural_verbs.rs:387-397`, `:709-716`): `GraphSubstrate` stores an edge as one bare `f64` strength, "gives a hyperedge no attributes at all" — widening it is a named Constitution-III.7 state-hash decision, not an oversight. Separately, the frozen system's "attribute absent vs. attribute present with any value" distinction (Python dict `in`) may not have a BSL analog at all if declared fields are always-present post-load — a content-modeling question independent of the storage gap. |
| Phase 2b predicate evaluation + transition (`_legacy.py:613-684`, `484-555`) | **BLOCKED — edge-attribute read+write (Slice 2) + no nested-map field type + no dynamic field-name lookup** | Three independent gaps stack here: (1) reading `edge_mode` and writing the transitioned `edge_mode` both need the same `edges`/`update-edge` lane as above; (2) `contradiction_fields`/`field_derivatives` are Python `dict`/`dict-of-dict` node attributes — `deffield`'s closed vocabulary has no map type at all, distinct from (and additional to) the already-known enum-storage gap; (3) the frozen interpreter looks up a node attribute BY THE STRING VALUE held in `condition.field` (e.g. whatever `"exploitation"` happens to be at that table row) — a dynamic/reflective field access BSL's `:field` binding (a literal declared-name binding) does not appear to support at all. Gap (3) is structurally different from a storage-type problem: even with map-typed fields landed, "look up the field named by this runtime string" is a different capability than "read this literal named field." A port that UNROLLS the generic interpreter into 16 concrete rules (each referencing a literal field name, e.g. `social_class/exploitation`) sidesteps gap (3) entirely — a content-modeling decision for the eventual dossier, not resolvable here. The `select-max`-shaped priority selection (item 5, §4) is a favorable, already-landed structural match once the surrounding reads/writes are solved. |
| Phase 2c aspect reversal (`_check_aspect_reversal`, `_legacy.py:797-858`) | **BLOCKED — edge-attribute read+write (Slice 2)** | `_dominant_party` read/write hits the identical `update-edge`/`EdgeRef` gap as Phase 2a/2b. The `wealth` node read is otherwise a plain, portable comparison (no arithmetic beyond `>`/`<`) — this row's ENTIRE blocker is the edge-attribute lane, nothing else. Also carries its own undeclared-field content-modeling question (§3: `_dominant_party` is on no Pydantic model at all, unlike Territory's every field). |
| Phase 3 CO-OPTIVE breakdown (`_handle_co_optive_breakdowns`, `_legacy.py:744-794`) | **BLOCKED — same non-graph accumulator gap as Phase 0/1** | Pure host-side `persistent_data` manipulation; zero graph reads/writes. The `co_optive_breakdowns` list itself (Phase 2b→Phase 3 same-`step()` communication) is a plain local Python variable, tighter even than persistent_data — a BSL analog would need either a single monolithic rule spanning what the frozen code splits into three functions, or some other same-tick, same-rule-execution communication channel; see the D116 note below. |
| 16-transition threshold table (data, not a computation) | **PORTABLE NOW (as `defconst`s)** | Frozen at Python import time via a bare `GameDefines()` — never live-read from `defines.yaml` in the frozen system either (§4 finding). A `defconst`-per-threshold port loses zero moddability relative to the frozen behavior; needs the bare-`0.0`/non-integer-literal workaround already precedented in landed packs. Gated entirely on the surrounding edge-lane blocker being solved first — this row describes the DATA's portability, not the computation's. |
| `co_optive_suppression_rate` / `latent_release_multiplier` (defines) | **PORTABLE WITH D-RECORD** | Live-read from `services.defines` every tick — ordinary `[0,1]`/`[1,5]` coefficients, straightforward `defconst`s once (and only once) Phase 1/Phase 3's accumulator-representation D-record is settled; moot in isolation since the pathway they feed is itself blocked. |
| `default_transition_priority` (define) | **NOT-A-PACK** | Dead — read nowhere in `src/babylon` (grep-confirmed). No port action; a WS4-style adjudication candidate (bless-as-reserved or retire), same category as Territory's dead AUTO-mode defines. |
| `d2f_dt2`/`laplacian`/`regime` metric branches + `_REGIME_CODES` (`_legacy.py:509-521`, `473`) | **NOT-A-PACK** | Structurally unreachable given the shipped 16-entry table (verified: only `"value"`/`"df_dt"` appear as `metric=` literals anywhere in `_build_transitions()`, and `CompoundPredicate`/`PredicateCondition` are constructed nowhere else in `src/`). A faithful 16-concrete-rule port naturally omits this dead interpreter capability; if the Director instead wants a byte-for-byte generic-interpreter transcription regardless of reachability, the `dialectical_regime` graph-level attribute read is a FOURTH storage location (neither node, edge, nor the query-lane's node/neighbor surface) with no BSL representation named anywhere in the current surface — flag as an open question, not resolved here. |

**RESERVED-LINE.** The entire `_build_transitions()` table (`_legacy.py:92-447`) — the 16 permissible
transitions, their predicates, thresholds, priorities, and human-readable `description` strings — IS
the system's theoretical content, not incidental data. Several transitions cite Constitutional
sections and named theorists directly in their `description` field: `shared_enemy_alliance`
("Shared enemy produces alliance (I.15 united front)", line 333) and `co_optive_breakdown`
("CO-OPTIVE breakdown: material basis erodes (George Jackson)", line 402). The `EdgeMode`/
`ContradictionCharacter` enum vocabularies themselves encode Constitution I.14's antagonistic/
non-antagonistic-contradiction distinction. Any transcription decision that could shift WHICH
material conditions trigger WHICH dialectical transition (the field-flattening decision in the
Phase 2b blocker row above, the enum→BSL encoding, any dead-code pruning of the regime pathway)
touches this content and should be described to the Director for the eventual dossier, never
decided unilaterally by a porting agent.

**D116/Q14 note (rule-sharing across positions, not adjudicated here — named per the task brief).**
The frozen `step()` is one tight, single-function sequencing across all three named phases: Phase
2b appends to `co_optive_breakdowns` (a plain local variable) and Phase 3 consumes it in the SAME
call, with no persistent_data round trip in between. If the eventual port models Phases 1/2/3 as
THREE SEPARATE `.bscn` rules at ONE shared anchor position (mirroring the frozen code's own phase
comments), it needs the not-yet-landed "two rules at one position share pre-state" capability
(D116, open) for Phase 3 to see Phase 2's same-tick breakdown list. Modeling the whole `step()` as
ONE monolithic rule sidesteps this specific gap (at the cost of matching the frozen code's own
functional decomposition less closely) — a structural decision for the eventual dossier.

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_edge_transition_system.py` | 409 | **Primary conformance-oracle candidate.** Six test classes: `TestEdgeTransitionSystemBasic` (name, no-registry forced-predicate firing, edges-without-mode skipped), `TestEdgeTransitionStateMachine` (EXTRACTIVE→ANTAGONISTIC transition, event emission, prohibited-transition non-firing), `TestContradictionCharacterFlag` (preserved vs. default-stamped), `TestAspectReversal` (event emission), `TestCoOptiveMechanics` (df/dt suppression, breakdown event), `TestRegimePredicateMetric` (ordinal evaluation, graph-attr read, absent→None) — this last class directly exercises the dead-relative-to-shipped-table `"regime"` metric branch via hand-constructed `CompoundPredicate`/`PredicateCondition` objects, not through `_build_transitions()`. Every fixture is hand-built `BabylonGraph` state (`social_class` nodes with explicit `contradiction_fields`/`field_derivatives`), never harvested from a canonical scenario. |
| `tests/unit/engine/laws/test_law_edge_transition.py` | 314 | **Property-based behavioral-contract laws** (Hypothesis). Four laws pinned with explicit source-line citations in the module docstring: L1 state-machine closure (`_VALID_TRANSITIONS` membership), L2 max-priority selection (proven against the shipped default table, not a live-defines injection — the module docstring's own "Caveat" section documents the same import-time-freeze fact this inventory independently re-derived in §4), L3 CO-OPTIVE suppression accumulator monotonicity/non-negativity, L4 latent-release amplification + accumulator clearing. Exactly the behavioral-contract layer CLAUDE.md's testing philosophy asks for — a strong conformance-oracle candidate independent of bit-exactness. |
| `tests/integration/test_field_topology_integration.py` | 171 | Three-system integration test: `ContradictionFieldSystem → FieldDerivativeSystem → EdgeTransitionSystem` run together across ticks, on a hand-built "Detroit metro" scenario (`_make_detroit_metro_graph`) — imports `DefaultFieldRegistry` explicitly, meaning it likely wires the registry path (unlike production's `_step_from_oppositions`), making it the one place `"immiseration"`/`"imperial_rent"`-driven transitions might actually be exercised. A strong pipeline-level conformance-oracle candidate, but note its field-population path diverges from production's live path (§5). |
| `tests/property/invariants/test_edge_mode_trajectory.py` | 205 | Property-based INV-009/spec-055 trajectory-legality test: Predicate A (synthesized N-event trajectory), Predicate B (observed end-to-end trajectory via `SimulationEngine`), Predicate C (final mode is a legal `EdgeMode`, folded into every read helper). Uses `edge_mode_trajectory_strategy()` from the sibling strategies file. A structural/invariant conformance-oracle candidate (never-invalid-transition-pair), not a behavior-value oracle. |
| `tests/property/strategies/edge_mode_evidence.py` | 73 | Hypothesis strategy generator only (no assertions) — `evidence_event_strategy()`/`edge_mode_trajectory_strategy()`, drawing from `_FIELDS = ("exploitation", "imperial_rent", "immiseration")` and all 4 metrics including the dead `d2f_dt2`/`laplacian`. Supportive infrastructure for the property tests above, not itself a test. |
| `tests/unit/dialectics/test_edge_mode_category.py` | 87 | **Pure data/graph-theoretic law test** over `_TRANSITION_MAP`/`_VALID_TRANSITIONS` as data (BFS reachability between modes) — never calls `step()` at all. Explicitly documents (module docstring) a design-vs-code discrepancy it deliberately tests the TRUE shipped behavior for, not a naive universal — a schema/topology conformance-oracle candidate for the transition table's own graph structure, independent of the system's execution. |
| `tests/unit/engine/test_system_order.py` | 300 | Whole-engine ordering test (3 hits for `EdgeTransitionSystem` among many systems) — confirms tick position, not this system's behavior specifically. Schema-level, not a conformance oracle for this system. |
| `tests/unit/engine/test_event_conversion.py` | 1774 | Generic event-payload→observer conversion tests; 4 hits are exactly the 4 `EventType`s this system emits, exercised with hand-built payloads to test the CONVERSION pipeline, not this system's own logic. Not a conformance oracle for `EdgeTransitionSystem`; relevant only for confirming the 4 emissions' payload shapes are independently pinned elsewhere. |
| `tests/unit/config/test_observability_spine.py` | 415 | Grep-confirmed zero direct hits for `EdgeTransitionSystem` — appeared in the initial file list via broader `edge_transition`/event-name matches; not a conformance oracle for this system. |
| `tests/integration/test_ooda_detroit.py`, `tests/unit/ooda/test_layer3.py`, `tests/unit/ooda/test_ooda_system.py` | 431 / 287 / 492 | Grep-confirmed zero `EdgeTransitionSystem` hits despite appearing in the initial `edge_transition`-keyword file list (likely matching on `EdgeMode`/`edge_mode` used elsewhere in the OODA/electoral pipeline, e.g. `electoral.py`'s CO_OPTIVE coupling). Not conformance oracles for this system. |

**qa:regression byte-gate coverage.** `tools/regression_test.py::graph_content_hash` (924-964)
hashes every node/edge attribute of the `WorldState→graph` projection, so any change to this
system's outputs on any canonical scenario WOULD be caught — but per §5's verified dormancy finding,
that coverage is **real for nothing**: no canonical scenario seeds an `edge_mode`-bearing edge
(`tools/regression_scenarios.py:2856-2866`, a declared coverage gap), and Phase 0's one
unconditional write (`persistent_data["latent_contradictions"] = {}`) is not part of the graph hash
at all (it lives in `persistent_data`, not on `graph`). A port's conformance fixtures must be entirely
hand-built — `test_field_topology_integration.py`'s "Detroit metro" pattern is the closest existing
precedent, though even it diverges from production's live field-population path (§5, §7).

---

## Adjudication (2026-08-12)

Adjudicated against the current dev tree (`9324482f`). This is the most carefully-sourced blocker
table in the batch — every §6 Rust anchor was re-checked and every one holds verbatim. The two
places it goes wrong are on the **Python** side, in the cross-system channel table it claims is
exhaustive, and in one internally inconsistent verdict cell. Three corrections and four
confirmations.

1. **CORRECTION — `ElectoralSystem` is not the only production `edge_mode` writer; the OODA
   action lane is a second, same-tick one this inventory never searched.** `resolve_negotiate`'s
   coalition arm writes `edge_mode=EdgeMode.CO_OPTIVE.value` on an Organization→Organization
   `TRANSACTIONAL` edge, minting the edge if absent
   (`src/babylon/engine/actions/negotiate.py:157-170`). It is wired into the single-source-of-truth
   dispatch table at `src/babylon/engine/actions/__init__.py:67`
   (`ActionType.PROPOSE_ALLIANCE: resolve_negotiate`), declared eligible at
   `src/babylon/ooda/action_eligibility.py:34`, and dispatched **in-tick** by `OODASystem` @14.0
   via `resolve_player_action` (`src/babylon/engine/systems/ooda.py:317-326`). §5's channel-table
   claim ("the ONLY other production writer besides this system itself is
   `ElectoralSystem`@17.45") and §1's file map are both wrong. The same resolver writes a **fifth**
   edge attribute in this family, `co_optive_dependence` (`negotiate.py:162,169`), which §3's type
   inventory does not list.
2. **CORRECTION — the DoctrineSystem channel is SAME-TICK on that path, not cross-tick as §5
   states.** §5 reasons that `doctrine.py:197` @14.7 necessarily reads "**last tick's** value"
   because this system runs at 21.0. That is true of this system's writes and of Electoral's
   (@17.45), and false of the negotiate write at 14.0, which Doctrine reads 0.7 positions later
   the **same** tick. `negotiate.py:145-150` states the coupling explicitly: the CO_OPTIVE stamp
   "is precisely the mode `doctrine._practice_env` counts into `CO_OPTIVE_SHARE` — so every
   coalition entry walks the org one step toward the liquidationism absorbing state." A live
   same-tick channel into RESERVED-LINE doctrine content, denied by the table.
3. **CORRECTION — Phase 0's "PORTABLE WITH D-RECORD" contradicts its own detail cell and the
   sibling inventories' treatment of the identical shape.** The cell's own text — "no BSL construct
   (node field, edge field, graph attr, or otherwise) represents host-side persistent_data of this
   shape" — is a BLOCKED finding; a D-record records a *deviation*, not an absent representation.
   Confirmed against dev: the R9 §3.6 carrier-node ruling that would serve graph-scope state
   depends on `the`, which is `("the", "slice 2")` in `UNSERVED_EXPRESSION_HEADS`
   (`rust/crates/babylon-bsl/src/evaluator.rs:506`), and `(domain :graph)` does not execute at all
   — zero `RuleDomain::Graph`/`loaded.domain` in `babylon-bsl/src/tick.rs` or
   `babylon-tick/src/lib.rs`, and `subject_type_of` refuses outright a rule with no `:field`
   binding: "slice 1 runs rules over a population, not over the graph as a whole"
   (`tick.rs:158-181`). The sibling `wealth-distribution` inventory rates the identical
   graph-scope-state shape BLOCKED. The one route that might eventually serve `latent` — flattening
   `latent[node][field]` into two declared `social-class/latent-*` fields, legitimate because §5
   proves the live key space is exactly `{exploitation, atomization}` — is itself gated on the edge
   lane, since the write is reached *from* a CO_OPTIVE edge. Phase 0/1/3 are a second, independent
   BLOCKED gap, not a footnote to the first.
4. **CONFIRMATION (verdict-preserving) — the deep dormancy conclusion survives correction 1
   intact.** The org→org edge `resolve_negotiate` stamps also joins node types that never carry
   `contradiction_fields`/`field_derivatives` (social_class-only:
   `contradiction_field.py:110`, `field_derivative.py:120`), so the vacuous-`concessions_withdrawn`
   result holds on the newly-found path exactly as on Electoral's. The inventory's headline
   dormancy claim is right; only its enumeration of the paths it holds over was incomplete.
5. **CONFIRMATION — the `concessions_withdrawn` hand-trace is exactly right, verified at source.**
   `_legacy.py:429-445`: a **single**-condition predicate, `field="imperial_rent"`,
   `metric="value"`, `operator="lt"`, `scope="target"`, `threshold=et.concessions_withdrawn_threshold`
   — whose default is **1.0** (`src/babylon/config/defines/consciousness.py:434-439`), priority 7.
   With `imperial_rent` never populated in production, `_evaluate_condition`'s silent-`0.0` value
   branch makes `0.0 < 1.0` true unconditionally. This is the sharpest finding in the report and
   it holds under adversarial check.
6. **CONFIRMATION — every §6 Rust anchor, re-verified against dev tip.** `UNSERVED_EXPRESSION_HEADS`
   lists `edges`/`edge-between`/`the` as "slice 2" (`evaluator.rs:503-512`); `SERVED_QUERY_HEADS` is
   exactly `["nodes", "neighbors"]` (`:527`); `eval_field_of`'s doc states "an `EdgeRef` referent is
   unreachable today (no expression form produces one yet; slice 2 mints `EdgeKey`)" (`:1190-1195`);
   `update-edge` is refused with the identical `GraphSubstrate`-keys-one-`f64` message at BOTH
   sites (`structural_verbs.rs:387-397` and `:709-716`), and the module doc names the widening a
   Constitution-III.7 state-hash decision "escalated rather than silently absorbed" (`:16-27`).
   The claim that these were verified by the reader rather than copied from the brief is borne out.
7. **CONFIRMATION — the import-time defines freeze and the 16/17 reconciliation.** `_build_transitions`
   calls a bare `GameDefines()` at `_legacy.py:96` (§1's ":94" is the import line; §4's ":92-96"
   range is right), `_TRANSITIONS = _build_transitions()` at module scope `:450`, while
   `_co_optive_suppression` (`:708`) and `_handle_co_optive_breakdowns` (`:763`) read
   `services.defines` live — so the asymmetry is real and the `defconst` conclusion is favourable
   as stated. Direct count confirms **16** `name=` sites; `_VALID_TRANSITIONS` is that 16-pair set
   plus the hand-added `(ANTAGONISTIC, ANTAGONISTIC)` self-loop at `:461-462`, i.e. 17 — which
   also proves the 16 pairs are pairwise distinct.

**FINAL VERDICT: BLOCKED — sustained, on the same primary gap (the Slice-2 edge-attribute lane
plus the declared `GraphSubstrate` storage gap that refuses `update-edge` at every execution
path), with two amendments: (i) the `persistent_data` `latent_contradictions` accumulator is
promoted from "PORTABLE WITH D-RECORD" to a SECOND independent BLOCKED gap, so Phases 0/1/3 are
blocked twice over; (ii) §5's cross-system channel table must be restated to include the
OODA/`resolve_negotiate` same-tick `edge_mode` write path (position 14.0) and its same-tick
DoctrineSystem read (14.7). Neither amendment moves the verdict; both change what the eventual
dossier has to account for.**

**INADEQUATE-COVERAGE (scoped).** §5 asserts an exhaustive set of production `edge_mode` writers
on the strength of a grep restricted to `src/babylon/engine/systems/*.py`. The Action phase's
resolvers live in `src/babylon/engine/actions/` and run **inside** the tick via
`OODASystem`@14.0 — they are engine code, not client code. A re-read must add
`src/babylon/engine/actions/negotiate.py` (the whole file), `src/babylon/engine/actions/__init__.py`
(`VERB_RESOLVERS`, `:58-68`), and `src/babylon/engine/systems/ooda.py:310-330` (the in-tick
dispatch), and must re-run every "grep-confirmed the only …" claim in §5 across
`src/babylon/engine/` rather than `engine/systems/` alone.
