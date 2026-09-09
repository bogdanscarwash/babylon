# DecompositionSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `DecompositionSystem` (369 lines, tick position 11.0, Material Base) is a
small, self-contained, single-entity-search-and-split system: it looks up the Labor Aristocracy
node by `role`, waits a defines-configured delay after a `SUPERWAGE_CRISIS` event (or fires early
if LA is about to starve), then splits its population/wealth 15%/85% into a `CARCERAL_ENFORCER`
and an `INTERNAL_PROLETARIAT` node (creating them on demand if absent) and emits
`CLASS_DECOMPOSITION`. Its core operation — `_find_entity_by_role`, a linear search over every
`SOCIAL_CLASS` node for the one whose `role` (an 8-member enum) matches a target — hits a **real,
newly-identified BSL gap**: the landed query lane (`nodes`/`fold`/`select-max`) can only read a
queried element's field via `field-of`, and `field-of` is explicitly refused for `:enum-type`
fields (D102) — so no landed mechanism reads an *arbitrary* node's enum field inside a query
predicate today. The Territory-precedent workaround (int-ordinal encoding instead of a real
`defenum`) sidesteps this and remains available. Separately, the system's whole temporal-staggering
logic lives in `TickContext.persistent_data` — a Python side-channel with no graph representation
and, so far as this agent found, no BSL analogue at all — making it the load-bearing cross-system
channel to `ControlRatioSystem` (not any graph attribute). The system is **fully dormant on all
five canonical `qa:regression` scenarios** (a declared, PRE-EXISTING gap in
`tools/regression_scenarios.py`), though a dedicated 5200-tick scenario test proves the mechanism
live off the canonical estate.

**Verdict:** PORTABLE WITH D-RECORDS, contingent on the same enum→int-ordinal workaround Territory
used (native `defenum` does not compose with cross-node search) and a persistent_data→graph-field
re-modeling for the delay/latch state; no libm hazards; two distinct `EventType` emissions, both
unpinnable pending WS1 (#502).

**UPDATE (2026-08-17) — the port itself has landed. Verdict: PORTED (with one named omission).**
`docs/superpowers/plans/2026-08-17-decomposition-controlratio-port.md` (Tasks 0-4, PR A #614
MERGED) ships `decomposition.bsl`'s six rules (`p01-la-census`, `p02-superwage-warning`,
`p03-trigger`, `p04-enforcer-intake`, `p05-ip-intake`, `p06-la-deactivate`) against
`decomposition-conformance.bscn` (six social classes + one singleton `carceral-register`
INSTITUTION carrier) plus its delay-path companion world, `decomposition-delay-conformance.bscn`.
**The one named omission: the frozen `_create_target_entity`/`_derive_entity_id` on-demand
creation branch (spec-071) is NOT ported.** `add-node` is refused at content load
(`DEFERRED_SHAPE_VERBS`, `structural_verbs.rs:1723-1730`) — a MINTING verb the collect-then-apply
pre-state repair does not yet serve. Every conformance world this train ships pre-seeds its own
`CARCERAL_ENFORCER`/`INTERNAL_PROLETARIAT` targets instead; a world lacking either pre-seeded
target is UNPORTED for that branch, not equivalent to the frozen engine's on-demand creation.
Register row **D167**; follow-on **#562** (the structural-verb execution surface / T5, Program 29)
is the placeholder-id design that would eventually close this gap. This inventory's own §6 row
("Node creation on demand — PORTABLE WITH D-RECORD") and its Adjudication correction 6 (the
`:const`-literal escape "not expressible as stated") are both SUPERSEDED by the omission, not
merely narrowed — the port does not attempt any id-collapsing escape at all, it omits the branch.

**This inventory's §6 "BLOCKED — enum-field query predicate" row for `_find_entity_by_role` is
also SUPERSEDED, not resolved as recommended.** D102 discharged (Task 1, P27 territory-port
train) before this train needed the int-ordinal `SocialRole` workaround this row (and its
Adjudication's confirmation 1) recommends; the port declares the REAL `enum SocialRole` instead,
closed by two OTHER independent laws (D138's compound-fold-body refusal + `E-TYPE-044`) that force
the census onto a per-node, subject-gated reformulation (register row **D165**, its companion
**D169** records the rejected int-ordinal alternative and the reasoning). The **Task-0 dossier**
(`reports/decomposition-controlratio-bsl-surface-facts-2026-08-17.md`) independently re-verified
every BLOCKER-1 through BLOCKER-5 claim TRUE at the byte and additionally found this inventory's
own survey-row grading ("row 11.0 … none blocking") WRONG — it never read `structural_verbs.rs`
and so never discovered the `add-node` refusal above (Task-0 dossier §3 item 1); the dossier's
citation corrections (§2.1, §2.3) are what let Task 1 cite `typecheck.rs`/`evaluator.rs` at their
CURRENT (not the plan's stale) line numbers.

**Every other frozen-code oddity this inventory names transcribes exactly as filed**, with the
`bool`→0/1-int write-boundary corrections (Adjudication items 4-5) and the additive/overwrite
asymmetry (§4 item 6) both landed verbatim in `p04`/`p05`/`p06`; the two-site docstring-drift
finding (§4 item 8) is WIDENED by this port's own archaeology — `CarceralDefines`' OWN class
docstring (`territory.py:265-267`) carries a second, independently-stale "2.33:1"/"No crisis"
arithmetic computed off the same stale 30/70 split (register row **D173**). Full record: ADR212
(`ai/decisions/ADR212_decomposition_controlratio_port_handoff.yaml`); register rows D165-D174.

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/decomposition.py` | 369 | **The target.** `DecompositionSystem`, `_find_entity_by_role`, `_derive_entity_id`. Self-contained — no calls into `formulas/` or `domain/` (only imports: `kernel.event_bus`, `kernel.tick_partition`, `models.entity_registry`, `models.enums`, `kernel.graph_protocol`/`kernel.services` (TYPE_CHECKING only), `kernel.system_base`, `kernel.system_protocol` — decomposition.py:17-27). |
| `src/babylon/config/defines/territory.py` | 624 (whole module; `CarceralDefines` is lines 248-315) | Coefficient source — `CarceralDefines` Pydantic model. **Lives in the `territory` config module**, not a dedicated `carceral.py` — an odd-but-real home confirmed by `_assembler.py:170`. |
| `src/babylon/data/defines.yaml` | (carceral block: lines 291-300) | Player-editable coefficient values, `carceral:` section. |
| `src/babylon/config/defines/_assembler.py` | (relevant: 117, 170, 336) | Wires `CarceralDefines` into `GameDefines.carceral`. |
| `src/babylon/models/entities/social_class.py` | 522 | `SocialClass` Pydantic entity — field types/domains for every attribute the system reads/writes (`wealth`, `population`, `active`, `role`, `subsistence_threshold`, `s_bio`, `s_class`, `inequality`, `county_fips`). |
| `src/babylon/models/enums/social.py` | 211 | `SocialRole` (8-member `StrEnum`, decomposition.py's core discriminant) — declaration order at social.py:35-43. |
| `src/babylon/models/enums/events.py` | 234 | `EventType.SUPERWAGE_CRISIS`/`EventType.CLASS_DECOMPOSITION` (lines 91-92). |
| `src/babylon/models/enums/topology.py` | 253 | `NodeType.SOCIAL_CLASS` (line 62). |
| `src/babylon/models/entity_registry.py` | 203 | `CORE_BOURGEOISIE_ID` ("C003", decomposition.py's payer_id), plus the full `ROLE_TO_ENTITY_ID` map — in the bridged/legacy canonical topology every role decomposition touches has a **fixed, known node id** (LA=C004, enforcer=C005, proletariat=C006). |
| `src/babylon/kernel/event_bus.py` | 289 | `Event` (frozen dataclass), `EventBus.publish`/`.get_history`/`.clear_history`. |
| `src/babylon/kernel/tick_partition.py` | 31 | `TickPartition.MATERIAL_BASE`. |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase` — DecompositionSystem inherits it but uses **none** of its helpers (`_read`, `_write_clamped`, `_publish`, `_wrap_graph`); it calls `graph.update_node`/`services.event_bus.publish` directly and reads dicts with bare `.get(...)`. |
| `src/babylon/kernel/system_protocol.py` | 42 | `ContextType` (forward-ref alias to `TickContext`). |
| `src/babylon/kernel/graph_protocol.py` | 494 | `GraphProtocol` — `add_node`/`get_node`/`update_node`/`query_nodes` signatures (lines 62-98, 258-276). |
| `src/babylon/topology/graph.py` | 1033 (relevant: `add_node` 165-181, `get_node`/`update_node` 651-670) | Concrete `BabylonGraph` — plain dict merge, **no type coercion or quantization at tick time** (same fact Territory's inventory recorded). |
| `src/babylon/topology/adapters/query_mixin.py` | 146 | `QueryMixin.query_nodes` (lines 34-68) — iterates `self._graph.nodes`, which is insertion-ordered (deterministic, III.7-compliant; confirmed via `graph.py`'s module docstring on `_ids`/`_adj` insertion-ordered mirrors). |
| `src/babylon/engine/context.py` | 113 | `TickContext.persistent_data: dict[str, Any]` — the **entire** state channel the system's delay/latch logic depends on. |
| `src/babylon/engine/simulation_engine.py` | (relevant: 328-401) | `_SYSTEM_CLASSES` — confirms tick position 11.0 and full neighbor ordering. |
| `src/babylon/engine/systems/economic.py` | 836 | `ImperialRentSystem` (position 9.0) — the production emitter of `SUPERWAGE_CRISIS` this system's docstring names as its trigger; also writes `SOCIAL_CLASS.wealth`/`subsistence_threshold` on the LA node same-tick, upstream of position 11.0. |
| `src/babylon/engine/systems/control_ratio.py` | 247 | `ControlRatioSystem` (position 12.0) — the **sole** downstream reader of this system's `persistent_data` output (`_class_decomposition_tick`), plus a reader of the `CARCERAL_ENFORCER`/`INTERNAL_PROLETARIAT`/`LUMPENPROLETARIAT` population/organization this system's writes make live. |
| `src/babylon/models/types.py` | 337 | `Currency`/`Probability`/`Gini` — `Annotated[float, ...]` domains for `wealth`, `subsistence_threshold`, `s_bio`, `s_class` (Currency, `[0,∞)`), `inequality` (Gini `[0,1]`). |

**Reference BSL packs / docs read for format and precedent** (all read in full or in the cited
ranges):
- `rust/crates/babylon-tick/content/scenarios/organization-foundation.bscn` (67 lines, full) and
  `rust/crates/babylon-tick/content/rules/organization.bsl` (32 lines, full) — the one landed,
  real usage of `defenum`/`deffield ... enum` + `:field` self-scoped read + `=` guard (ADR195/196).
- `rust/crates/babylon-tick/tests/enum_arithmetic_e2e.rs` (115 lines) — confirms `set` is the only
  coherent update-op on an enum field; `add`/`sub`/`scale` refuse at load (D118).
- `rust/crates/babylon-tick/tests/query_lane_e2e.rs` (413 lines) — the landed Territory query-lane
  e2e; every field read inside a `fold`/`select-max` body uses `field-of it <field>` over
  **non-enum** territory fields, never a foreign-type `:field` binding — i.e. the only *exercised*
  in-query field-read idiom on dev is `field-of`.
- `rust/crates/babylon-tick/content/rules/lifecycle.bsl` (409 lines; cited range 336-360) — a
  **landed pack that itself sidesteps the enum-storage gap** with the older int-ordinal convention
  ("`LegitimationClassification` has no BSL representation... Encoded as this pack's own
  convention: 0 = STABLE, 1 = UNSTABLE, 2 = CRISIS") even though `defenum` now exists — direct
  precedent for the same choice this port would need to make.
- `docs/reference/bsl-language.rst` — §2.4 (`<cond>`/`<query>` predicates, lines ~734-750), §2.5
  (Bindings — `:field` self/foreign-type scoping rules, lines 820-919), §2.6 (Queries, lines
  939-1046), §2.8 (Effects/structural verbs incl. `add-node`, lines 1326-1360), §2.10 (Element
  accessors — the five-form closed accessor list and the `field-of` enum deferral, lines
  1805-1894), §2.13 addendum (the `enum` type row + `defenum`/`defvocabulary` grammar + D101/D102,
  lines 2170-2286).

## 2. COMPUTATION CATALOG (execution order, `step()` decomposition.py:110-223)

### Step 1 — LA death-proximity check (decomposition.py:143-159)
- **(a)** Look up the Labor Aristocracy entity; compute whether it is "approaching" subsistence
  (within 2 ticks' consumption) or already below it.
- **(b)** `la_approaching_death = la_wealth < subsistence + (2 * consumption) and la_pop > 0`
  (decomposition.py:155-156); `la_about_to_die = la_wealth < subsistence and la_pop > 0`
  (decomposition.py:158-159), where `consumption = s_bio + s_class` (decomposition.py:153). Pure
  read — no write here.
- **(c) Reads:** `SOCIAL_CLASS.wealth` (default 0.0), `.subsistence_threshold` (default 0.0),
  `.population` (default 0), `.s_bio` (default 0.0), `.s_class` (default 0.0) — all on the LA node
  found via `_find_entity_by_role(graph, SocialRole.LABOR_ARISTOCRACY)` (active-only, the default
  `include_inactive=False`).
- **(d) Writes:** none.
- **(e) Defines:** none.
- **(f) Events:** none (yet).

### Step 2 — SUPERWAGE_CRISIS detection + early-warning emission (decomposition.py:161-197)
- **(a)** If a crisis hasn't already been recorded, scan the event-bus history for
  `SUPERWAGE_CRISIS` events and record the earliest tick; if none exists yet but LA is
  "approaching death", **emit** an early-warning `SUPERWAGE_CRISIS` itself, one tick before the
  fallback would otherwise force decomposition (the code's own stated purpose: "ensures at least 1
  tick gap between the events" — decomposition.py:142).
- **(b)** `superwage_tick = min(e.tick for e in crisis_events)` over `services.event_bus.get_history()`
  filtered by `e.type == EventType.SUPERWAGE_CRISIS.value` (decomposition.py:167-174); cached into
  `persistent["_superwage_crisis_tick"]`. The self-emitted fallback event
  (decomposition.py:180-195) carries `payer_id=CORE_BOURGEOISIE_ID` ("C003"), `receiver_id=la_id`,
  `desired_wages=0.0`, `available_pool=0.0`.
- **(c) Reads:** `services.event_bus.get_history()` (see §5 for the two-driver semantics
  discrepancy this depends on); `persistent["_superwage_crisis_tick"]`.
- **(d) Writes (side-channel, not graph):** `persistent["_superwage_crisis_tick"] = tick`.
- **(e) Defines:** none (the threshold is hardcoded as "2 * consumption", a bare literal — see §4
  item 1).
- **(f) Events:** conditionally emits `EventType.SUPERWAGE_CRISIS`.

### Step 3 — delay gate (decomposition.py:199-211)
- **(a)** Decide whether to execute decomposition **this** tick: either LA is already dying
  (fallback, immediate) or the configured delay has elapsed since the crisis tick.
- **(b)** `should_decompose = la_about_to_die or (superwage_tick is not None and tick >= superwage_tick + delay)`
  (decomposition.py:200-211), `delay = services.defines.carceral.decomposition_delay`.
- **(c) Reads:** `persistent["_superwage_crisis_tick"]`, `la_about_to_die` (Step 1's local).
- **(d) Writes:** none.
- **(e) Defines:** `carceral.decomposition_delay` (default 52, `[0,520]` — defines.yaml:298,
  `CarceralDefines` territory.py(defines):298-303).
- **(f) Events:** none.

### Step 4 — `_execute_decomposition` (decomposition.py:217, body 263-369)
- **(a)** Re-find LA; if absent or already-zero population, no-op. Split LA's population and
  wealth 15%/85% (defines-configured) between a `CARCERAL_ENFORCER` and an `INTERNAL_PROLETARIAT`
  target entity (creating either on demand if not already present in the graph, spec-071),
  deactivate LA, and emit `CLASS_DECOMPOSITION`.
- **(b)** `enforcer_pop_gain = int(la_population * enforcer_fraction)`,
  `proletariat_pop = int(la_population * proletariat_fraction)`,
  `enforcer_wealth_gain = la_wealth * enforcer_fraction`,
  `proletariat_wealth = la_wealth * proletariat_fraction` (decomposition.py:298-301). Enforcer
  write is **additive**: `population=current_pop + enforcer_pop_gain`,
  `wealth=current_wealth + enforcer_wealth_gain` (decomposition.py:327-332). Proletariat write is
  **an overwrite, not additive**: `population=proletariat_pop`, `wealth=proletariat_wealth`
  (decomposition.py:334-336) — see §4 item 6, a verbatim frozen-code asymmetry. LA itself:
  `graph.update_node(la_id, active=False)` (decomposition.py:339) — **`wealth`/`population` are
  never zeroed or otherwise touched on the LA node itself**; it stays inactive but keeps its
  pre-decomposition wealth/population values in the graph (see §4 item 7).
- **(c) Reads:** `SOCIAL_CLASS.population`/`.wealth` on LA; `.population`/`.wealth` on the
  enforcer target (for the additive branch); `services.defines.carceral.enforcer_fraction`/
  `.proletariat_fraction`.
- **(d) Writes:** `SOCIAL_CLASS.population`, `.wealth`, `.active` on the enforcer node;
  `.population`, `.wealth`, `.active` on the proletariat node; `.active` on the LA node; and, if
  either target is absent, a brand-new `social_class` node via `add_node` (`_create_target_entity`,
  decomposition.py:225-261 — full model-complete payload: `id`, `name`, `role`, `active=False`,
  `population=0`, `wealth=0.0`, `county_fips` (inherited from LA), `subsistence_threshold`, `s_bio`
  (default 0.01), `s_class`, `inequality` — all inherited from LA's own values where present).
- **(e) Defines:** `carceral.enforcer_fraction` (default **0.15**, `[0.05,0.50]`),
  `carceral.proletariat_fraction` (default **0.85**, `[0.50,0.95]`) — defines.yaml:295-296. **Note
  the module docstring (decomposition.py:4-6) says "30%... 70%" — stale relative to the shipped
  default; transcribed verbatim below as a defect, not corrected** (see §4 item 8).
- **(f) Events:** emits `EventType.CLASS_DECOMPOSITION` unconditionally on success
  (decomposition.py:342-368), payload includes `source_class`, `source_population`,
  `source_wealth`, both fractions, both transferred-population/wealth dicts,
  `trigger_event=EventType.SUPERWAGE_CRISIS.value`, `narrative_hint`.

**Node-creation id derivation — `_derive_entity_id`** (decomposition.py:36-50, called from
`_create_target_entity`): `seed = int(digits) % 1000` from `base_id`'s digits (0 if none), then a
**bounded 1000-iteration search** `for i in range(1000)` for the first free
`f"C{(seed + offset + i) % 1000:03d}"`, falling back to the untested offset id if all 1000 are
taken (`# pragma: no cover`). `_ENFORCER_ID_OFFSET=700`, `_INTERNAL_PROLETARIAT_ID_OFFSET=800`.
Deterministic (no `hash()`), but structurally a **bounded free-slot search over graph state**, not
a pure expression of the LA's id — see §6.

**Events emitted by the whole system: two distinct `EventType` values** —
`SUPERWAGE_CRISIS` (early-warning path, decomposition.py:182) and `CLASS_DECOMPOSITION`
(decomposition.py:344), confirmed by grep (no other `EventType`/`.publish(` in the file).

## 3. TYPE INVENTORY

Runtime storage note (identical fact Territory's inventory recorded):
`BabylonGraph.update_node`/`.add_node` (`topology/graph.py:165-181,660-670`) are plain dict
merges with **no type coercion or quantization mid-tick**; `Currency`/`Probability`/`Gini`'s
`SnapToGrid` (1e-5/1e-6 grid) applies only at Pydantic-model instantiation (scenario seed /
`WorldState` round-trip), never inside a System's `step()`.

| Attribute | Node type | Python model type | Domain | Category |
|---|---|---|---|---|
| `role` | SOCIAL_CLASS | `SocialRole` (`StrEnum`, 8 members) | closed set, social.py:35-43 | **Enum discriminant** — the system's entire lookup key |
| `wealth` | SOCIAL_CLASS | `Currency` (`Annotated[float, ge=0.0]`) | `[0.0, ∞)` | **unbounded real, money-semantic** |
| `population` | SOCIAL_CLASS | `int`, `ge=0` | `≥ 0` | integer |
| `active` | SOCIAL_CLASS | `bool` | `{T,F}` | boolean latch, monotone one-way in this system (LA: True→False only; targets: False→True only) |
| `subsistence_threshold` | SOCIAL_CLASS | `Currency` | `[0.0, ∞)` | unbounded real, money-semantic — **never written by any System** (scenario-seed static, grep-confirmed) |
| `s_bio`, `s_class` | SOCIAL_CLASS | `Currency`, `ge=0.0` each | `[0.0, ∞)` | unbounded real — also never written by any System |
| `inequality` | SOCIAL_CLASS | `Gini` (`Annotated[float, ge=0.0, le=1.0]`) | `[0.0, 1.0]` | unit-interval — read-and-copy only (inherited into a created target, never computed here) |
| `county_fips` | SOCIAL_CLASS | `str \| None`, `pattern=r"^\d{5}$\|^$"` | 5-digit FIPS or `""`/`None` | optional string, read-and-copy only |
| `name` | SOCIAL_CLASS | `str`, `min_length=1` | any non-empty string | required-on-create; the system synthesizes it: `f"{role.value} (decomposed from {la_id})"` |
| `carceral.decomposition_delay`, `.enforcer_fraction`, `.proletariat_fraction` (defines) | — | `int` / `float` / `float` | `[0,520]` / `[0.05,0.50]` / `[0.50,0.95]` | tick-count + two unit-interval coefficients |
| `persistent["_superwage_crisis_tick"]`, `["_class_decomposition_tick"]` | — (side-channel, not a graph attr) | `int \| None` | any tick number | run-scoped ephemeral state, **no BSL analogue found** (§5) |
| `persistent["_decomposition_complete"]` | — (side-channel) | `bool` | `{T,F}` | one-shot latch, same channel |

**Enum discriminant flag — the load-bearing one for this system.** `role` is not merely present
(as Territory's `profile`/`territory_type` were, read only against `self`); it is the **search
key** of `_find_entity_by_role`, which scans every `SOCIAL_CLASS` node for the one whose `role`
equals a target member. §3.1's `enum` type row (bsl-language.rst) makes this representable as a
declared `deffield ... enum SocialRole` field, landed and exercised once
(`organization-foundation.bscn`/`organization.bsl`) — but every landed use of it is **self-scoped**
(`:field organization/kind` reads the rule's own anchor node, compared in a `when` guard). See §6
for why that precedent does not close this system's actual need.

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `float`)/native `int`; grep-confirmed zero
`exp`/`log`/`sigmoid`/`pow` calls anywhere in decomposition.py. **Zero libm-nondeterminism hazard**
for this system, same as Territory. Shapes, in execution order:

1. **Bare-literal threshold, no defines backing:** `la_wealth < subsistence + (2 * consumption)`
   (decomposition.py:155) — the `2` is a hardcoded magic number, not read from `GameDefines`. Not
   in `carceral.*` or any other defines category (grep-confirmed). A verbatim frozen-code oddity —
   transcribe as a bare `2c` constant with a D-record noting it has no config-file backing, rather
   than inventing a define for it (port-as-is law).
2. **Additive time-window comparison:** `subsistence + (2 * consumption)` where
   `consumption = s_bio + s_class` — two adds, one multiply, all `Currency`-typed operands treated
   as plain `Real` (no scale-op needed; none of these are `:field`-sourced `Currency` in the
   BSL-typed sense once ported, since the port would declare these fields `int`/`Real` per the
   Territory/Metabolism precedent for money-like fields — see §6).
3. **Real→Int demotion (×2):** `int(la_population * enforcer_fraction)` and
   `int(la_population * proletariat_fraction)` (decomposition.py:298-299) — truncating cast, both
   operands non-negative (`la_population: int ≥ 0`, fractions `> 0`), so **trunc ≡ floor** —
   expressible today via the landed `floor` intrinsic (`DECLARABLE_INTRINSICS = ["exp","log","floor"]`,
   `declarations.rs:110`), same resolution class as Territory's post-#489 correction.
4. **Plain multiply (currency × unit-interval):** `la_wealth * enforcer_fraction`,
   `la_wealth * proletariat_fraction` (decomposition.py:300-301) — the same "Currency-typed operand
   is actually `:field`-sourced `Value::Real`, so the #500 scale-op doesn't apply" shape Territory's
   D-1 class already names for `rent_level * rent_spike_multiplier` and Metabolism's
   `entropy_factor`; here the coefficient's domain (`[0.05,0.50]`/`[0.50,0.95]`) is inside `[0,1]`
   (unlike Territory's out-of-`[0,1]` `rent_spike_multiplier`), so this is the *milder* member of
   the D-1 class — a bare-scaled-Int/Real workaround still applies, ADR183.
5. **Additive accumulation (enforcer target only):** `current_pop + enforcer_pop_gain`,
   `current_wealth + enforcer_wealth_gain` (decomposition.py:329-330) — one add each, order-trivial
   (single term).
6. **Asymmetric write shape — a verbatim frozen-code defect, not a computation per se.** The
   enforcer target's write is **additive** (`current + gain`) while the proletariat target's write
   is a **flat overwrite** (`population=proletariat_pop`, `wealth=proletariat_wealth`, discarding
   whatever the proletariat node held before — decomposition.py:334-336 vs. 327-332). On every
   topology this agent could verify, the proletariat target is seeded at `population=0, wealth=0.0`
   and stays untouched (dormant, `active=False`) until this exact write, so the two shapes are
   *observationally* equivalent on the canonical estate today — but they are not the same
   computation, and port-as-is law requires transcribing both shapes faithfully rather than
   silently unifying them into one idiom.
7. **A write that does NOT happen — LA's own `wealth`/`population` are never zeroed.**
   `graph.update_node(la_id, active=False)` (decomposition.py:339) touches only `active`. LA keeps
   its full pre-decomposition `wealth`/`population` values in the graph, merely flagged inactive.
   Combined with `creates_value: ClassVar[bool] = True` and the class's own comment ("Default-deny
   while audit pending; flip to `False` once internal redistribution is proven sum-preserving",
   decomposition.py:105-108) — this is the frozen system **itself documenting that its
   wealth-conservation property is unproven**. A naive downstream `fold sum` over
   `SOCIAL_CLASS.wealth` that does not gate on `active` would double-count: LA's stale wealth is
   still physically present, on top of the amount already copied into the two targets. Verbatim
   transcription obligation, not a bug to fix at port time.
8. **Docstring/default mismatch — 30/70 vs. 15/85.** The module docstring
   (decomposition.py:1-6, class docstring 90-96) states "30% ... 70%"; the shipped
   `CarceralDefines` default is `enforcer_fraction=0.15`, `proletariat_fraction=0.85`
   (defines.yaml:295-296, matching `docs/concepts/terminal-crisis.rst`'s stated 15%/85% theory
   text). The **behavior** (what actually executes) uses 15/85; the **comment** is stale. Port-as-is
   law: transcribe the behavior (15/85 default), D-record the docstring discrepancy verbatim rather
   than silently "fixing" the comment in a way that would misrepresent what the frozen code does if
   the reader trusted the docstring instead of the Field default.
9. **Clamp implementations:** none. Unlike Territory, this system writes no `[0,1]`-bounded field
   directly — `population`/`wealth` are `≥0`-only (never explicitly clamped; `int(...)` truncation
   and non-negative multiplicands keep them non-negative by construction, not by an explicit
   `max(0, ...)`).

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 11.0** (decomposition.py:102), confirmed against `_SYSTEM_CLASSES`
  (`simulation_engine.py`): `... → DispossessionEventSystem (10.0) → DecompositionSystem (11.0) →
  ControlRatioSystem (12.0) → MetabolismSystem (13.0) → ...`. `ImperialRentSystem` runs at 9.0 —
  two positions earlier, same tick.
- **Same-tick upstream read: `ImperialRentSystem` (9.0) writes `SOCIAL_CLASS.wealth` on the WAGES-edge
  target** (the LA node in the canonical topology) via several `update_node(edge.target_id,
  wealth=...)` calls (`economic.py:295-297,387,516`). DecompositionSystem's Step 1/Step 4 both read
  `la_data.get("wealth")` **fresh this tick**, after ImperialRentSystem's write — a real same-tick
  channel. `ImperialRentSystem` is also the class docstring's named trigger ("Must run AFTER
  ImperialRentSystem (which emits SUPERWAGE_CRISIS)", decomposition.py:98) — confirmed:
  `economic.py:462-487` emits `EventType.SUPERWAGE_CRISIS` under its own pool-exhaustion condition,
  **independently of** DecompositionSystem's own early-warning emission (Step 2 above) — **two
  separate production emitters of the same event type**, one upstream-same-tick, one inside this
  system itself.
  `subsistence_threshold`/`s_bio`/`s_class` are **never written by any System** (grep-confirmed
  across `src/babylon/engine/systems/*.py` for `subsistence_threshold=`/`"s_bio"`/`"s_class"` as
  `update_node` keyword targets) — static scenario-seed data throughout the run.
- **The `services.event_bus.get_history()` read (Step 2) depends on a driver-specific EventBus
  lifecycle that is NOT uniform across production paths — a fact worth flagging precisely.**
  `game/session.py:1592` calls `bus.clear_history()` immediately before each `run_tick`, so on that
  driver `get_history()` returns only the current tick's events (matching the code's own comment,
  decomposition.py:214-216: "EventBus is recreated each tick (ephemeral)"). The canonical
  `qa:regression`/headless driver (`engine/headless_runner/runner.py:1249`) constructs **one**
  `EventBus()` for the **entire run** and this agent found no `clear_history()` call anywhere under
  `engine/headless_runner/` — so on that driver, `get_history()` accumulates every event from every
  prior tick, not just the current one. The logic still behaves correctly either way (it takes
  `min(e.tick for e in crisis_events)` and caches the result once found, so accumulation vs.
  per-tick clearing produces the same `superwage_tick`), but the code's own comment overclaims
  "recreated each tick" as if it were universally true, when it is a property of one of two
  production drivers. UNVERIFIED beyond these two call sites: this agent did not trace whether
  `WorldStateBridge`/the `Simulation` class used by `tests/scenarios/test_carceral_equilibrium.py`
  clears history per tick or not (search run: `rg -n 'clear_history' src/babylon` found exactly the
  three hits cited above, none in `game/simulation` or the bridge).
- **Writes consumed downstream — split between graph-mediated and side-channel-mediated, and they
  are NOT the same shape:**
  - **Graph-mediated:** `SOCIAL_CLASS.population`/`.wealth`/`.active` on the enforcer/proletariat/LA
    nodes. `wealth`/`population`/`active` are each read by a broad set of Material-Base-and-later
    systems generically (grep: `wealth` read in 11 other system files, `population` in 10,
    `active` in 15 — `allegiance.py`, `contradiction.py`/`contradiction_field.py`, `economic.py`,
    `ideology.py`, `market_scissors.py`, `production.py`, `struggle.py`, `survival.py`,
    `vitality.py`, `electoral.py`, `epistemic_horizon.py`, `lifecycle.py`, `metabolism.py`,
    `community.py`, `policy.py`, `reactionary.py`, `solidarity.py`), but **none of these reads is
    live on the canonical estate** because the enforcer/proletariat nodes stay `active=False,
    population=0` until this system's dormant decomposition actually fires (see below). Two
    role-specific readers stand out: `wealth_distribution.py:64,66` classes `CARCERAL_ENFORCER`
    into wealth-bracket tier 2 and `INTERNAL_PROLETARIAT` into tier 3 (Program 21 Phase-1 shadow
    axis), and `community.py:49-50` maps both roles to the `"PROLETARIAT"` community tier — both
    also gated dormant by the same fact.
  - **`ControlRatioSystem` (12.0), the one system architecturally coupled to this one, reads it
    through `persistent_data`, not the graph.** `control_ratio.py:128-134`:
    `decomposition_tick = persistent.get("_class_decomposition_tick"); if decomposition_tick is
    None: return`. This is the **entire** gate on whether `ControlRatioSystem` does anything at all
    — "the step() body returns on its first guard clause every tick" whenever this system hasn't
    successfully decomposed (declared verbatim in `tools/regression_scenarios.py:2825-2830`,
    quoted below). `ControlRatioSystem` separately reads `SOCIAL_CLASS.role`/`.active`/
    `.population`/`.organization` for `CARCERAL_ENFORCER`/`INTERNAL_PROLETARIAT`/
    `LUMPENPROLETARIAT` (`control_ratio.py:53-85`), which is the graph-mediated half of the same
    coupling, equally dormant.
- **Context/service usage with no BSL equivalent — the system's central architectural fact.** The
  entire temporal-staggering mechanism (the docstring's own stated purpose: "delays
  CLASS_DECOMPOSITION by the configured number of ticks... ensures phase staggering") lives
  **entirely** in `context.persistent_data`, a plain Python `dict[str, Any]`
  (`engine/context.py:49`) that is **not part of `WorldState`/the graph substrate** at all. Three
  keys: `_superwage_crisis_tick`, `_decomposition_complete`, `_class_decomposition_tick`. This
  agent searched `docs/reference/bsl-language.rst`'s full `<bind-src>` production
  (§2.5, line 827-834: `:field | :const | :metric | :tick | :year | :tick-of-year |
  :tick-in-cycle | :expr`) and found **no binding source that reads or writes an out-of-graph
  scratch/context value** — every declared way a BSL rule can persist state across ticks is a
  graph node/edge/hyperedge field (which *is* part of the substrate and therefore *does* survive
  tick-to-tick). This is a genuinely different shape from Territory's single `TickContext`
  finding (`displacement_mode`, a stateless per-run override) — here the side-channel **is the
  system's core state machine**, and it is also the **entire** cross-system channel to
  `ControlRatioSystem` (there is no graph-attribute alternative already in use for that specific
  coupling). Not provably a hard blocker — the natural re-modeling is to store these as `int`/`bool`
  fields on a graph node (the LA node itself, or a dedicated singleton) instead of a Python dict —
  but it is a real re-architecture the port must do deliberately, not a like-for-like transcription.
- **Dormancy on canonical scenarios — CONFIRMED, and already independently declared in the estate's
  own coverage-gap ledger** (`tools/regression_scenarios.py`, `COVERAGE_GAPS_DATA`, verified at
  lines 2816-2823):

  > `"system": "DecompositionSystem"`, `"reason": "SUPERWAGE_CRISIS never fires (neither
  > ImperialRentSystem's pool-exhaustion path nor this system's own approaching-death
  > early-warning path) within 150 ticks in any of the five scenarios, so CLASS_DECOMPOSITION
  > correspondingly never fires"`, `"remediation": "a longer-horizon or more austerity-calibrated
  > scenario that actually exhausts the imperial rent pool or starves the labor aristocracy"`.

  And immediately following it (lines 2824-2830), `ControlRatioSystem`'s gap entry names this
  system's dormancy as its own root cause verbatim: `"gated entirely behind
  persistent_data['_class_decomposition_tick'], set only by a successful DecompositionSystem run —
  which never happens in these five... the step() body returns on its first guard clause every
  tick"`, `"remediation": "resolves automatically once the DecompositionSystem gap is closed"`. The
  same ledger's dense-column audit (lines 504-517, 518-531) independently cross-validates the gap
  for `C005_wealth`/`C005_effective_wealth` via the same citation chain. **This is a
  PRE-EXISTING, already-documented gap** — this agent's own investigation corroborates it (no new
  discovery) but did not stop there.
  - **The mechanism is provably live off the canonical estate.** `tests/scenarios/test_carceral_equilibrium.py`
    (436 lines) runs a dedicated `MAX_TICKS = 5200` (100-year) horizon against a purpose-built
    `create_imperial_circuit_state()` fixture (`tests/scenarios/conftest.py:45`, **not** the same
    factory `tools/regression_scenarios.py`'s canonical scenarios use) with explicit "NO player
    organization" calibration, and asserts `SUPERWAGE_CRISIS → CLASS_DECOMPOSITION →
    CONTROL_RATIO_CRISIS → TERMINAL_DECISION` fire in that order. So the code is not dead in the
    sense of unreachable — it is reachable only outside the 52-150 tick canonical byte-gate
    horizon, on a fixture built specifically to reach it. A port's conformance oracle will need an
    equivalently hand-built `.bscn` fixture, exactly as Territory's inventory anticipated for its
    own dormant phases.
  - **`_create_target_entity`'s `add_node` branch liveness is genuinely unresolved by this agent
    and is flagged UNVERIFIED rather than asserted either way.** decomposition.py's own comment
    (lines 303-306) says "The bridged canonical world seeds no CARCERAL_ENFORCER /
    INTERNAL_PROLETARIAT entity, so without this the enforcer branch no-ops" — but the `_legacy.py`
    scenario factory `tools/regression_scenarios.py`'s canonical scenarios actually use **does**
    pre-seed both (`engine/scenarios/_legacy.py:370-386,504-505`: `CARCERAL_ENFORCER_ID`/
    `INTERNAL_PROLETARIAT_ID` present, `active=False, population=0`). This agent did not trace
    whether "the bridged canonical world" in the decomposition.py comment refers to a *different*
    seeding path (the spec-065 `WorldStateBridge`/county-scale hydration used by
    `headless_runner`) than the `_legacy.py` test-fixture factories — search run: `rg -n
    'CARCERAL_ENFORCER|INTERNAL_PROLETARIAT' src/babylon/engine` found only the `_legacy.py` and
    `decomposition.py`/`control_ratio.py` hits already cited; the bridge's own hydration code was
    not read. Either way, both branches are additionally gated dead by the tick-position-11.0
    dormancy above on the five canonical scenarios specifically.

## 6. BLOCKER ASSESSMENT

| Computation | Verdict | Detail |
|---|---|---|
| Step 1 — LA death-proximity check (decomposition.py:143-159) | **BLOCKED — enum-field query predicate** (same root cause as row 3 below) | Reading `wealth`/`subsistence_threshold`/`population`/`s_bio`/`s_class` is trivial once the LA node is found; the blocker is entirely in *finding* the LA node (see row 3) — this row inherits it. |
| Step 2 — SUPERWAGE_CRISIS early-warning emission (decomposition.py:177-197) | **PORTABLE WITH D-RECORD** (once row 3 is resolved) | Plain threshold `if` + `emit`; the bare `2` literal (§4 item 1) needs a `2c` constant with a "no defines backing" D-record note. `emit` itself lands with a real observable only once WS1 (#502) gives event emissions a TickReport-visible ledger — until then this is unpinnable by any conformance oracle, same as every other system's `emit`. |
| Step 3 — delay gate (decomposition.py:199-211) | **PORTABLE NOW** | `carceral.decomposition_delay` is a plain `int` `[0,520]` define — trivial `c`-suffixed `defconst`; `tick >= superwage_tick + delay` is ordinary int arithmetic and comparison, both already-landed primitives, **contingent only on `_superwage_crisis_tick` having a graph-field home** (the persistent_data re-model, §5). |
| `_find_entity_by_role` — the LA/enforcer/proletariat lookup (decomposition.py:53-86) | **BLOCKED — enum-field query predicate (new finding, not covered by any existing Q-item this agent found)** | The operation is "scan every `SOCIAL_CLASS` node, return the first whose `role` field equals a target `SocialRole` member." §2.6's `(nodes NodeType/SOCIAL_CLASS <node-pred>)` accepts a `<cond>` predicate over the iterated element `it` — but the **only** accessor that reads an arbitrary queried element's field is `field-of` (§2.10's closed five-form accessor list: `field-of`, `edge-between`, `the`, `metric-of`, `membership-field-of` — none of the other four reads a plain node field), and `field-of` is **explicitly refused for `:enum-type`-declared fields** ("this section does not extend §2.10's `field-of` accessor... to enum-declared fields," D102, bsl-language.rst ~line 2273-2280). `:field` bindings — the only other way to read an enum field (§2.13's own enum row: "read through a `:field` binding exactly as any other node field is") — are scoped to the rule's own `self` anchor (§2.5 C1 ruling), not to an arbitrary iterated `it`; the one landed real-world usage of a foreign-type `:field` binding inside a fold body (`query_lane_e2e.rs`) is never exercised for enum fields, and this system's search is exactly the shape (scan-all-of-a-type-by-a-field-value) that pattern does not cover for `field-of`-refused fields regardless. **Deviation available and precedented:** decline the native `enum`/`defenum` type for `social-class/role` and use the same int-ordinal workaround Territory's inventory named for `profile`/`territory_type` (and `lifecycle.bsl` uses live, today, for `LegitimationClassification` — `field-of` is unrestricted for `int`-typed fields) — this reopens `field-of`-based filtering inside `nodes`/`select-max`/`fold` predicates at the cost of the semantic fidelity `defenum` would have bought. D-record: "SocialRole → int-ordinal (8 members, order per social.py:35-43), NOT `defenum`, specifically to preserve query-predicate filterability — a generalizable tension this port's D-record should name explicitly, since `role` is read the identical way by Production/Reactionary/MarketScissors/WealthDistribution/Struggle/ControlRatio/Community/EpistemicHorizon, all of which face the same choice once ported." |
| Node creation on demand — `_create_target_entity`/`_derive_entity_id` (decomposition.py:225-261, 36-50) | **PORTABLE WITH D-RECORD** | `add-node` is a landed structural verb (`<verb> ::= ... "(" "add-node" <enum-ref> <expr> <field-init>* ")"`, §2.8) — the model-complete field-init payload transcribes directly. `_derive_entity_id`'s 1000-iteration collision search has no BSL loop construct to express (no `while`/`loop`/recursion, §2.7's closing statement) — but on every topology this agent could verify (`entity_registry.py`'s fixed `ROLE_TO_ENTITY_ID` map: LA=C004, enforcer=C005, proletariat=C006), the target id is a **known constant**, so the collision search provably never advances past `i=0`. D-record candidate, Metabolism-D-2-style ("provably uniform"): declare the target id a `:const`/literal rather than porting the search loop, with the same "unverified beyond the canonical topology" caveat §5 already raised about which world seeds what. |
| Population/wealth split arithmetic (decomposition.py:298-301, 327-336) | **PORTABLE WITH D-RECORD** | `int(pop * frac)` — expressible via the landed `floor` intrinsic (non-negative operands, trunc ≡ floor). `wealth * frac` hits the same D-1-class hazard Territory/Metabolism already named (Currency-typed-but-`:field`-sourced-as-`Value::Real` operand) — same bare-scaled-Real/Int workaround, ADR183. The additive-vs-overwrite asymmetry (§4 item 6) must be transcribed as two *different* update-op shapes (`add` for enforcer, `set` for proletariat, in §2.8's closed four-op vocabulary), not unified. |
| LA deactivation, no wealth/population zeroing (decomposition.py:339) | **PORTABLE NOW** | A single `update-node ... active (set #f)`. The absent zeroing (§4 item 7) is itself faithfully portable — it's a *non*-write, nothing to express beyond not writing it — but the D-record must note the same conservation caveat the frozen code's own comment raises, so a future `fold sum` over wealth in the ported pack does not silently "fix" what the frozen system leaves unfixed. |
| CLASS_DECOMPOSITION event emission (decomposition.py:341-368) | **PORTABLE WITH D-RECORD**, same class as Step 2 | `emit` lands; the payload's nested dicts (`population_transferred: {to_enforcer, to_proletariat}`, `wealth_transferred: {...}`) are richer than `<payload-item> ::= "(" <symbol> <expr> ")"` (flat key-value pairs, §2.8) supports directly — the nesting would need flattening (e.g. `population-transferred-to-enforcer`, `population-transferred-to-proletariat`) as its own D-record. Unpinnable pending WS1 regardless. |
| `TickContext.persistent_data` state machine (`_superwage_crisis_tick`, `_decomposition_complete`, `_class_decomposition_tick`) | **PORTABLE WITH D-RECORD, architecturally the largest one** | No `<bind-src>` in §2.5's closed set reads an out-of-graph scratch value — every cross-tick state channel in BSL is a graph field. Re-model as `int`/`bool` fields on a designated node (the LA node itself, or a dedicated singleton `:ceiling 1` node reachable via `the`) rather than a Python dict. This is the same re-modeling `ControlRatioSystem`'s own eventual port will need for the identical channel, since it is the **sole** reader of `_class_decomposition_tick` — the two ports are not independent and should be designed together. |

**Not exercised at all by this system:** no edge reads/writes of any kind (`query_nodes`/`get_node`/
`update_node`/`add_node` only — grep-confirmed zero `query_edges`/`add_edge`/`update_edge` calls),
so none of Slice 2's edge-attribute-read gap binds here; no hyperedge/metric usage, so Slice 3
doesn't bind either.

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_la_decomposition.py` | 533 | **Primary conformance oracle.** Exhaustive coverage of `DecompositionSystem.step()`: 15/85 split under shipped defines, event emission + narrative hint, no-decomposition-without-crisis, idempotency (only-once), missing-target creation (both enforcer and proletariat independently), wealth proportionality, exact-delay-boundary / one-tick-before-delay timing, zero-delay edge case, approaching-death early warning, about-to-die fallback overriding the delay, `int()` truncation confirmation, completion-flag reentry guard. |
| `tests/unit/engine/systems/test_decomposition_enforcer_creation.py` | 121 | Spec-071-specific: on-demand target creation, id pattern validity, created-target survives a `from_graph()`/`to_graph()` round-trip. Direct conformance-oracle candidate for the `_create_target_entity`/`_derive_entity_id` blocker row. |
| `tests/unit/engine/laws/test_law_decomposition_system.py` | 292 | **Property-based invariant contracts** (P27 Phase-0 backfill, explicitly written to trace file:line ranges): L1 bounded-population-split (never exceeds source, loses ≤1 unit per target to truncation), L2 monotone accumulation, plus no-op-after-completion, no-LA-writes-nothing, zero-population-writes-nothing. Exactly the behavioral-contract-law shape the BSL port's own conformance scenarios should re-prove independent of bit-exactness. |
| `tests/integration/mechanics/test_class_decomposition.py` | 399 | Multi-tick integration: normal SUPERWAGE_CRISIS→delay→CLASS_DECOMPOSITION path, fallback path, split-matches-defines, LA-deactivated-after, dormant-entities-activated-after, one-time idempotency, custom-fraction override. |
| `tests/unit/engine/systems/test_superwage_crisis.py` | 210 | Tests `ImperialRentSystem` (economic.py), **not** DecompositionSystem — the upstream trigger-event producer. Adjacent, not primary. |
| `tests/scenarios/test_carceral_equilibrium.py` | 436 | Long-horizon (5200-tick) narrative/phase-sequence scenario test spanning Decomposition + ControlRatio + the terminal-decision bifurcation together (the "70-Year Arc"/"null hypothesis" trajectory, `ai/carceral-equilibrium.md`'s successor `docs/concepts/carceral-equilibrium.rst`). Proves the mechanism live off the canonical estate (§5); a narrative/emergent-behavior test, not a per-computation conformance oracle. |
| `tests/unit/engine/systems/test_control_ratio.py` | 756 | Downstream `ControlRatioSystem` unit tests — adjacent, exercises the `persistent_data["_class_decomposition_tick"]` gate from the reading side. |
| `tests/unit/engine/laws/test_law_control_ratio.py` | 270 | Downstream `ControlRatioSystem` property-based laws — adjacent. |
| `tests/integration/mechanics/test_control_ratio_crisis.py` | 373 | Downstream `ControlRatioSystem` integration — adjacent. |
| `tests/unit/engine/test_system_order.py` | 300 | Pins the full 34-System tick ordering, including `"DecompositionSystem"` at its named position (lines 46, 74, 243) — a schema/ordering conformance test, not a math oracle. |

**qa:regression byte-gate coverage.** `tools/regression_test.py::graph_content_hash` (line 924)
hashes every node/edge attribute of the `WorldState→graph` projection on every canonical scenario,
so any change to `DecompositionSystem`'s outputs **would** be caught by the byte-identical hash
gate **if the system ever fired** — but per §5's confirmed dormancy, it never does within any
canonical scenario's horizon. Coverage is therefore **zero** for this system on the byte-gate
today; the primary conformance oracle for a port is `test_la_decomposition.py` +
`test_law_decomposition_system.py`, and any canonical-scenario-style `.bscn` fixture will need to
be hand-built to a longer horizon or harsher austerity calibration, exactly as
`tools/regression_scenarios.py`'s own declared remediation states.

---

## Adjudication (2026-08-12)

Adjudicated against the dev tree at `9324482f`. Four corrections, three confirmations. This
inventory's central finding is correct and is the best-reasoned enum analysis in the batch; the
corrections are all on the *effect* side, which it graded from the spec rather than from the
executor.

1. **CONFIRMATION — the D102 finding is real, is enforced, and this inventory is the one that got
   it right.** `field_of_node` (`rust/crates/babylon-bsl/src/evaluator.rs:1274-1292`) is indeed the
   only accessor that reads an arbitrary queried element's node field, and the enum gate sits in
   front of it — enforced at LOAD, not at eval, by `check_no_field_of_on_enum_field`
   (`rust/crates/babylon-bsl/src/typecheck.rs:246-280`, wired at `rule_pipeline.rs:297`) with the
   refusal text *"… not extended to enum-declared fields (§2.13, D102)"*. Spec text at
   `docs/reference/bsl-language.rst:2274-2284`; register row D102 at `:5681-5693`, deferred-not-
   forbidden exactly as quoted. The `:field`-is-self-scoped half is confirmed at the mechanism:
   `subject_type_of` derives the subject from the shared namespace of the rule's `:field` bindings
   and errors when they disagree — *"a field is a field OF self's node type"* (`tick.rs:159-182`),
   and `bind_field_value`'s `BslType::Enum` branch is the only one that renders a `Value::Enum`
   (`tick.rs:312-378`). The int-ordinal escape and its landed `lifecycle.bsl` precedent both hold.
   **This row adjudicates two sibling inventories in the same batch:** `ImperialRentSystem`'s §6
   grades `role` as "**PORTABLE** … no D-record needed beyond declaring the 8-member enum," and
   `ControlRatioSystem`'s Phase-2 row files a D-record to "declare `social-class/role enum
   SocialRole`" — both would break on this gate for the identical foreign-node read shape. Both
   have been corrected in their own adjudications, citing this one.

2. **CONFIRMATION — every frozen-code oddity transcribed verbatim checks out.** Docstring
   "30% … 70%" (`decomposition.py:3-5`) against shipped defaults `enforcer_fraction: 0.15` /
   `proletariat_fraction: 0.85` (`defines.yaml:295-296`); the additive enforcer write
   (`decomposition.py:325-332`, `population=current_pop + enforcer_pop_gain`) against the flat
   proletariat overwrite (`:334-336`, `population=proletariat_pop`); LA touched only as
   `graph.update_node(la_id, active=False)` (`:339`) with `wealth`/`population` never zeroed; the
   bare `2 *` in `la_wealth < subsistence + (2 * consumption)` (`:155`); `include_inactive=True`
   on the two target lookups (`:305,310`) against the LA lookup's active-only default (`:143`).
   The `_class_decomposition_tick` write is at `:223`, read by `ControlRatioSystem` at
   `control_ratio.py:128` — the Class-D coupling is exactly one key wide, as claimed.

3. **CONFIRMATION — dormancy, position and reserved-line surface.** Coverage-gap row verbatim at
   `tools/regression_scenarios.py:2817-2823`, with ControlRatio's dependent row at `:2825-2830`.
   Tick position 11.0 (`decomposition.py:102`) between `DispossessionEventSystem` (10.0) and
   `ControlRatioSystem` (12.0), against `_SYSTEM_CLASSES` (`simulation_engine.py:328-360`), whose
   sort by `position` IS `_DEFAULT_SYSTEMS` (`:376-378`). **No RESERVED-LINE surface flagged, and
   independently confirmed absent** — no doctrine content, no National Question parameter, no
   outcome-definition logic in this system. (Note for the eventual port: `ControlRatioSystem`, its
   Class-D partner, DOES carry one — ADR070/Program 19 rules its revolution-vs-genocide branch the
   explicitly-LAST system in the emergent-class-partition cutover. A joint Class-D port train
   inherits that flag even though this half does not raise it.)

4. **CORRECTION — "LA deactivation … **PORTABLE NOW** | A single `update-node … active (set #f)`"
   does not evaluate on current dev.** `numeric_write_value` (`structural_verbs.rs:1196-1234`) is
   the one funnel every node write crosses. It routes enum-declared fields to `enum_write_value`,
   accepts `Value::Real` (`:1220`) and `Value::Int` (`:1224`), refuses `Value::Currency` by name
   (`:1225-1230`), and falls through to *"cannot store {other:?} as a numeric node attribute"*
   (`:1231-1233`) for everything else — `Value::Bool` included. A `#f` literal evaluates to
   `Value::Bool` (`tick.rs:392`, `evaluator.rs:366`). The write is expressible only as `(set 0)`
   over a 0/1-encoded `int` field — the convention the two landed vitality packs adopted and
   documented in-file: *"0/1 rather than #t/#f: BSL has Bool (§3.1) but `deffield` has no bool"*
   (`content/scenarios/vitality-conformance.bscn:20`,
   `vitality-lifecycle-combined-conformance.bscn:34`; that comment's premise is itself stale —
   `"bool" => Ok(BslType::Bool)` at `declarations.rs:649` — but declaring `bool` changes nothing
   at the write boundary). Downgrade this row to **PORTABLE WITH D-RECORD**. The same correction
   binds the `active=True` writes in the split row (`decomposition.py:331,336`) and
   `_create_target_entity`'s `active=False` field-init (`:246-247`).

5. **CORRECTION — the `active` READ inside `_find_entity_by_role` (`decomposition.py:72-74`)
   carries the same unnamed encoding constraint, and it sits under Step 1's and Step 4's whole
   verdict.** No boolean is ever readable as a `Value::Bool`: `bind_field_value` returns
   `Value::Real(stored)` for every non-enum declared type (`tick.rs:312-327`) and `field_of_node`
   returns `Ok(Value::Real(value))` unconditionally (`evaluator.rs:1281-1291`). Downstream,
   `as_bool` refuses a Real where a `<cond>` is required (`evaluator.rs:1315-1320`) and
   `apply_equality` refuses it against `#t`/`#f` — *"equality is defined within one lane only"*
   (`evaluator.rs:1620-1628`). The predicate must be written `(= (field-of it social-class/active) 1)`
   over the 0/1 encoding. Fold this into the same D-record as correction 4 so the encoding is
   declared once and used on both sides.

6. **CORRECTION — the `_create_target_entity`/`_derive_entity_id` row's escape ("declare the
   target id a `:const`/literal rather than porting the search loop") is not expressible as
   stated: BSL node identity is substrate-minted, not author-named.**
   `GraphSubstrate::add_node(&mut self, node_type: &str) -> Result<NodeId, GraphError>`
   (`substrate.rs:80`) assigns the id; the `<expr>` operand of
   `(add-node <enum-ref> <expr> <field-init>*)` is a **rule-local declared NAME** —
   `let name = self.fresh_declared_name(id_expr, env)?; let id = graph.add_node(node_type)…;
   self.declared_nodes.insert(name, id);` (`structural_verbs.rs:852-864`) — usable only by later
   effects in the SAME rule firing, as its own pinning test says by name:
   `add_node_introduces_a_name_later_effects_can_use` (`structural_verbs.rs:1721-1728`). It is not
   a persistent cross-tick string id, and there is **no way to name an existing node by literal
   id** at all. So the frozen "find `C005`, else create it" pair (`decomposition.py:303-320`)
   cannot collapse to a constant: the *find* must go through the query lane (an `exists`/
   `select-min`-guarded `nodes` predicate over the int-ordinal `role`), and the D-record must
   record that the derived-id scheme has **no analogue**, rather than that it is provably
   `i=0`. The "provably uniform" reasoning still justifies dropping the 1000-iteration search —
   it does not justify a literal id.

7. **CORRECTION (smaller, but it points the port at the wrong door) — the `persistent_data`
   state-machine row proposes "a dedicated singleton `:ceiling 1` node reachable via `the`"; `the`
   does not evaluate.** It sits in `UNSERVED_EXPRESSION_HEADS` tagged `"slice 2"`
   (`evaluator.rs:505`); its singleton guard is LOAD-time only — `E-LOAD-043`, *"(the {row}) needs
   a declared :ceiling of exactly 1"* (`manifest.rs:51-53, 100-104`) — and **no landed content
   declares a `manifest` at all** (zero `manifest` hits across `rust/crates/babylon-tick/content/`).
   The route that does work on current dev is the one this system's Class-D partner verified
   independently: anchor the rule on the carrier type through its `:field` bindings and let
   `subject_type_of` derive the subject (`tick.rs:159-182`), so the rule fires once over a
   one-member population — no `the`, and no dependence on `(domain :graph)` being consumed
   (`run_tick` never reads `loaded.domain`; `RuleDomain::Graph` resolves at load only,
   `domain.rs:214`). Since the two ports must be designed together anyway, adopt that route here
   and cite the ControlRatio inventory's §6 row 1 for it.

**FINAL VERDICT: PORTABLE WITH D-RECORDS — UPHELD in kind, with a larger D-record set and two
rows downgraded.** The two D-records as filed are correct: (1) `SocialRole` → int-ordinal, NOT
`defenum`, to preserve `field-of` filterability under D102 — the best-argued row in this batch,
and the one that corrects two sibling inventories; (2) the `persistent_data` → graph-field
re-modeling, though via the `:field`-anchored carrier route rather than `the` (correction 7). Add:
(3) the boolean 0/1 `int` encoding, binding every `active` read AND write (corrections 4-5), which
downgrades "LA deactivation" from PORTABLE NOW to PORTABLE WITH D-RECORD; and (4) a node-identity
D-record recording that `add-node` names are rule-local and that no literal-id lookup exists, so
the find-else-create pair ports as a query-lane guard plus `add-node`, not as a constant
(correction 6). Zero libm hazards, zero edge reads or writes, no Slice-2/3 exposure, and full
dormancy on all five canonical scenarios all stand as filed. The Class-D obligation to port
jointly with `ControlRatioSystem` is confirmed from both sides.

**INADEQUATE-COVERAGE NOTE (narrow but load-bearing).** §1's Rust-side reading list is landed
content packs, two `babylon-tick` e2e test files, and `bsl-language.rst` — **no `babylon-bsl`
source**. That is exactly why the query/read side (graded off the spec and the landed packs) is
right while the effect side is wrong three times: `(set #f)`, the `add-node` id, and `the`. A
re-read must add `rust/crates/babylon-bsl/src/structural_verbs.rs` (`add_node` at :843-890,
`numeric_write_value` at :1196-1234, the `update-edge` refusal at :387-398),
`rust/crates/babylon-bsl/src/evaluator.rs:503-527, 1274-1292, 1315-1320, 1594-1632` (served/unserved
heads, `field-of`, the value lanes) and `rust/crates/babylon-graph/src/substrate.rs:80-145` (node
identity and the attribute surface). Reading the spec plus the landed packs is enough to grade a
*read*; it is demonstrably not enough to grade a *write*.
