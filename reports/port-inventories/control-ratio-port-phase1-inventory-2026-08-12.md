# ControlRatioSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** ControlRatioSystem (247 lines, position 12.0, MATERIAL_BASE) is a
pure read-and-emit system — grep-confirmed **zero graph writes** anywhere in the file — that
runs a four-phase crisis-gated state machine entirely over `context.persistent_data` (a
Python dict owned by the `Simulation` instance, never part of `WorldState`, never hashed, and
provably lost on save/load) rather than over graph state. Its readiness gate is keyed on a
flag written, the same tick, one position earlier, by `DecompositionSystem` (@11.0) — the two
systems form a single Class-D "crisis-gated" unit the 2026-08-10 gap survey rules must port
together. Its population/organization census (a filtered fold over `SOCIAL_CLASS` nodes) is
mechanically **portable now** under the Query-lane Slice 1 landing (ADR197, merged
2026-08-11) plus the landed enum-field machinery (ADR195/196) — a live correction to that
survey's own "(blocked)" tag, which predates both landings. The float surface is the
cleanest found in this estate so far (zero libm calls, zero clamps, zero `int()` casts), with
one sharp exception: a confirmed-reachable `float("inf")` special-value literal in an event
payload. The system is provably 100%-dormant on every `qa:regression` canonical scenario
(its own delay defines require ≥53 ticks to even reach the capacity check; the byte-gate runs
52) — the only live conformance oracle in the repository is a dedicated 5200-tick scenario
test.

**Verdict:** BLOCKED on Q6 (graph-scope state — `context.persistent_data` has no BSL
`<bind-src>`/write-verb) — with a concretely verified singleton-carrier D-record escape route
that also closes Q12 "for free" for this system — and must port together with
DecompositionSystem (Class D); the population/organization fold itself is PORTABLE NOW.

**UPDATE (2026-08-17) — the port itself has landed, jointly with DecompositionSystem exactly as
this inventory's Class-D verdict required. Verdict: PORTED (with one named omission).**
`docs/superpowers/plans/2026-08-17-decomposition-controlratio-port.md` (Tasks 5-8, PR B) ships
`control-ratio.bsl`'s four rules (`c01-prisoner-census`, `c02-publish-census`, `c03-crisis`,
`c04-terminal`) against four hand-built conformance worlds (the primary/genocide,
`-revolution-`, `-within-capacity-`, `-zero-enforcer-` scenarios) plus the joint
`carceral-arc-conformance.bscn` five-phase composition scenario proving the frozen
SUPERWAGE_CRISIS→CLASS_DECOMPOSITION→CONTROL_RATIO_CRISIS→TERMINAL_DECISION sequence reproduces
tick-for-tick against `carceral_arc_conformance.py`'s frozen mirror. Q6's own singleton-carrier
escape route (this section's confirmation 1) is the ONE the port uses, exactly as filed —
`(select-max (nodes NodeType/INSTITUTION) 1)`, never Decomposition's own inventory's `the`
proposal (corrected there citing this row). Register row **D166**.

**The one named omission: `float("inf")`'s ratio-key payload, not the state machine itself.**
`c03-crisis`'s BLOCKER-4 guard-split (already landed at Task 6) OMITS the `actual-ratio`/
`control-ratio` payload keys entirely when `enforcer-population == 0`, rather than encoding the
frozen `float("inf")` (`control_ratio.py:185`) — BSL's closed `<literal>` grammar has no infinity
form. Loud absence (III.11), not a fabricated number; the SAME payload minus those two keys is
still emitted. Register row **D171** item 4.

**Phase 2's own "portable now" grading is CORRECTED, not merely re-confirmed, per this
inventory's own Adjudication correction 4 — and further corrected by this train.** The
Adjudication already downgraded the census to "portable now under Slice 1 plus the int-ordinal
role encoding, NOT `deffield … enum`" (correction 4) with the boolean 0/1 `active` encoding
(correction 5) riding alongside. Both those corrections predate the Task-0 dossier confirming
D102 discharged (§2.1) — the port therefore declares the REAL `social-class/role enum SocialRole`
after all, closed instead by D138's compound-fold-body refusal + `E-TYPE-044` (register row
**D169**, the companion to Decomposition's own D165), and the `pop × organization` product this
inventory's own §2 computation catalog names (`_count_prisoner_population_and_org`, `:84`) is
forced onto `c01`'s per-node `:expr` binding by the identical law, not into `c02`'s carrier fold.
The `active` 0/1-int write-boundary correction (Adjudication item 5) transcribes verbatim.

**The RESERVED-LINE row (§6, "port-as-is, do not attempt the cutover") is HONORED exactly as
filed.** `control-ratio/c04-terminal` transcribes `_emit_terminal_decision` VERBATIM — same
threshold source, same `>=`, both outcomes, the same two prisoner roles — under ADR070/Program
19's explicit LAST ruling; the emergent-class-partition cutover stays Director-gated and open.
Register row **D174**; reaffirmed at **ADR208 R29/C-03** and register row 12 of #564. The
Task-0 dossier independently re-verified this inventory's Rust-side citations (§2.1-2.3) and
found two of its six ControlRatio line-pointers drifted (citation-precision only, no substance
change) — the corrected lines are what Tasks 5-7 cite.

Full record: ADR212 (`ai/decisions/ADR212_decomposition_controlratio_port_handoff.yaml`); register
rows D165-D174 (this pack's own header, `control-ratio.bsl:1-197`, carries the full transcription
detail as six in-file D-records, D-records 1-6 — the two-role prisoner set, the unconditional
census publication, the `<=` boundary, the guard-split emit, the numeric `outcome` encoding, and
the cross-pack byte-order inversion (D172's own content) — cross-referenced to the global register
above).

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/control_ratio.py` | 247 | **The target.** `ControlRatioSystem`, a single `step()` plus two module-level aggregation helpers and two `_emit_*` helpers. Zero calls into `formulas/`/`domain/`. **Zero graph writes anywhere in the file** — grep-confirmed no `update_node`/`add_node`/`remove_node` call exists in it. |
| `src/babylon/engine/systems/decomposition.py` | 369 | `DecompositionSystem` @11.0 — the SOLE producer of `persistent_data["_class_decomposition_tick"]` (`decomposition.py:223`) that gates every branch of ControlRatioSystem (`control_ratio.py:128-134`); same-tick, immediately-prior writer of `SOCIAL_CLASS.active`/`.population` on the CARCERAL_ENFORCER/INTERNAL_PROLETARIAT nodes ControlRatio counts that same tick (`decomposition.py:327-339`). Class D coupling (§5, §6) — the two systems must port together. |
| `src/babylon/config/defines/territory.py` | 624 (`CarceralDefines`: 248-315) | Coefficient source — `control_capacity`, `revolution_threshold`, `control_ratio_delay`, `terminal_decision_delay` (plus `enforcer_fraction`/`proletariat_fraction`/`decomposition_delay`, consumed by Decomposition, not ControlRatio). |
| `src/babylon/data/defines.yaml` | `carceral:` block, lines 293-301 | Player-editable coefficient values. |
| `src/babylon/config/defines/_assembler.py` | 411 (`carceral` field: 170, 336) | `GameDefines` assembly — wires `CarceralDefines` into `services.defines.carceral`. |
| `src/babylon/models/entities/social_class.py` | 522 (`role`:296-299, `organization`:355-358, `active`:380-383, `population`:406-410) | `SocialClass` Pydantic entity — field types/domains for every attribute ControlRatioSystem reads. |
| `src/babylon/models/enums/social.py` | 211 (`SocialRole`:12-44, `.coerce()`:45-57) | The 8-member `SocialRole` StrEnum, plus the centralized `.coerce()` classmethod ControlRatioSystem's own `_get_role` duplicates inline rather than calling (§3). |
| `src/babylon/models/enums/topology.py` | 253 (`SOCIAL_CLASS`:62) | `NodeType.SOCIAL_CLASS`. |
| `src/babylon/models/enums/events.py` | 234 (`CONTROL_RATIO_CRISIS`:93, `TERMINAL_DECISION`:94) | The two `EventType` members this system emits. |
| `src/babylon/models/types.py` | 337 (`Probability`:50-58) | `organization`'s underlying `Annotated[float, ge=0,le=1, SnapToGrid]` type — quantized only at Pydantic-model-instantiation time, never mid-tick (same non-quantization fact the Territory inventory established for `BabylonGraph.update_node`). |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase` — ControlRatioSystem uses **none** of its helpers (`_read`/`_write_clamped`/`_publish`/`_get_persistent_data`); it reads `context.persistent_data` and calls `services.event_bus.publish` directly. |
| `src/babylon/kernel/system_protocol.py` | 41 | `ContextType = "TickContext"` forward-ref alias. |
| `src/babylon/kernel/tick_partition.py` | 30 | `TickPartition.MATERIAL_BASE`. |
| `src/babylon/kernel/event_bus.py` | 288 (`Event` dataclass: 33-55) | `Event` — frozen dataclass, `type: str` (an `EventType` member coerces to its `.value` string at publish, matching Decomposition's own `.value` comparisons elsewhere in the estate). |
| `src/babylon/kernel/graph_protocol.py` | 494 (`query_nodes`:258-, `get_node`:77-86, `update_node`:88-98) | `GraphProtocol` signatures. |
| `src/babylon/kernel/services.py` | 88 (`ServicesProtocol`:23-53) | `services.defines`/`services.event_bus` surface. |
| `src/babylon/topology/adapters/query_mixin.py` | 146 (`query_nodes`:34-68) | Concrete `BabylonGraph.query_nodes` — a plain Python generator over `self._graph.nodes` (rustworkx insertion order, deterministic), no DB pushdown. This is the fold ControlRatioSystem's two helper functions hand-roll. |
| `src/babylon/engine/context.py` | 113 (`TickContext`:19-114) | `persistent_data: dict[str, Any]` — the field ControlRatioSystem's entire trigger logic is built on. |
| `src/babylon/engine/simulation_engine.py` | 611 (`_SYSTEM_CLASSES`:328-363, `TickContext` construction/sync: 564-580) | Confirms tick position 12.0, AND the exact mechanics by which `persistent_data` survives across ticks: a fresh `TickContext` every tick, seeded from and synced back to the **caller's** dict — never the graph, never `WorldState`. |
| `src/babylon/engine/simulation/_legacy.py` | 1074 (`class Simulation`:52, `_persistent_context` init:120, reset:986) | Confirms `_persistent_context` is `Simulation`-**instance** Python state, not `WorldState` — lost on save/load, never part of a hash. |
| `src/babylon/engine/scenarios/_legacy.py` | 1270 (`carceral_enforcer`:372-386, `internal_proletariat`:390-404) | `create_imperial_circuit_scenario` — the canonical factory that seeds BOTH prisoner-adjacent roles, dormant (`population=0`, `active=False`) — used, directly or via `defines_overrides`, by every entry in `tools/regression_scenarios.py`'s `SCENARIOS` dict bar `two_node`. |
| `tools/regression_test.py` | 1862 (`DEFAULT_MAX_TICKS`:81, tick loop:1054) | `DEFAULT_MAX_TICKS = 52` — the `qa:regression` byte-gate horizon (dormancy evidence, §5). |
| `tools/regression_scenarios.py` | 2925 | `SCENARIOS` dict — grep-confirmed no `carceral.*` override anywhere, no per-scenario tick-count override. |
| `ai/decisions/ADR070_emergent_class_partition.yaml` | 151 | Program 19 ruling: "ControlRatio (revolution-vs-genocide branch) LAST, no exception, only after low flip-count evidence, with a dedicated high-effort review" for the SocialRole→derived-class-cell adjudication cutover. RESERVED-LINE-adjacent (§6). |
| `reports/bsl-gap-analysis-2026-08-10.md` | 863 | The authoritative, dated BSL-blocker survey — row 60 (§1 table), the Q6/Q12/Q16 sections, the Class-D dormancy classification (~line 662). Central citation for §6; re-verified against the CURRENT dev tree, which has moved since 2026-08-10 (Query-lane Slice 1 / ADR197). |

**Rust-side files read for the §6 adjudication (all on current `dev`):**
- `rust/crates/babylon-bsl/src/tick.rs` (`subject_type_of`:159-182, `run_tick`:524-631) — the exact mechanism that (a) **refuses** a rule with zero `:field` bindings and (b) does **not** consult `RuleDomain::Graph` at execution time.
- `rust/crates/babylon-bsl/src/domain.rs` — confirms `RuleDomain::Graph` parses/resolves at LOAD time only (Q12's "UNDER-DETERMINED" status is really "parsed but unconsumed," true on current dev, not merely a 2026-08-10 snapshot).
- `rust/crates/babylon-bsl/src/declarations.rs` (lines 648-663) — confirms `bool` IS a valid `deffield` type today (corrects a stale in-repo comment; see §3).
- `rust/crates/babylon-tick/content/scenarios/query-lane-e2e.bscn` (header comment, lines 1-42) and `content/scenarios/organization-foundation.bscn` (lines 45,57,61) — precedent for (a) inventing a discriminator field purely to satisfy subject-type derivation, (b) enum declare+`=`-compare (`OrgKind`), (c) `social-class/organization int extensive` already declared once.

## 2. COMPUTATION CATALOG (execution order, `control_ratio.py:105-247`)

The gap survey's own framing (§6) is accurate and adopted here: this is one graph-scoped
four-phase state machine, not four independent per-node rules.

### Phase 1 — Readiness gate (`control_ratio.py:119-134`)
- **(a)** Before doing anything: check the system hasn't already resolved for this run
  (one-shot latch), then wait for `DecompositionSystem` to have fired, then wait an
  additional configured delay after that.
- **(b)** `if persistent.get("_terminal_decision_emitted"): return` (124-125, permanent
  one-way exit for the rest of the Simulation's life). `decomposition_tick =
  persistent.get("_class_decomposition_tick"); if decomposition_tick is None: return`
  (128-130). `delay = services.defines.carceral.control_ratio_delay; if tick <
  decomposition_tick + delay: return` (132-134).
- **(c) Reads:** `context.persistent_data["_terminal_decision_emitted"]` (bool|absent),
  `context.persistent_data["_class_decomposition_tick"]` (int|None — DecompositionSystem's
  write), `context.tick` (int).
- **(d) Writes:** none.
- **(e) Defines:** `carceral.control_ratio_delay` (int, default 52, domain `[0,520]` ticks —
  defines.yaml:299, `CarceralDefines` territory.py(defines):304-309).
- **(f) Events:** none.

### Phase 2 — Population/organization census (`control_ratio.py:53-85`, invoked 137-138)
- **(a)** Two independent folds over every `SOCIAL_CLASS` node: total active-enforcer
  population, and total active-prisoner population plus a population-weighted organization
  sum (for a later average).
- **(b)** `total = Σ attrs.get("population", 0)` over `query_nodes(SOCIAL_CLASS)` filtered to
  `active and role == CARCERAL_ENFORCER` (`_count_enforcer_population`, 53-62).
  `total_pop = Σ pop`, `org_sum = Σ pop * org` over the same query filtered to `active and
  role in {INTERNAL_PROLETARIAT, LUMPENPROLETARIAT}` (`_count_prisoner_population_and_org`,
  65-85).
- **(c) Reads:** `SOCIAL_CLASS.active` (bool, default `True` at both the read site and the
  model), `SOCIAL_CLASS.role` (via `_get_role`, 40-50), `SOCIAL_CLASS.population` (int, model
  default 1, **read default here is 0** — §3 divergence note), `SOCIAL_CLASS.organization`
  (`Probability`, model default 0.1, **read default here is 0.0** — same divergence pattern).
- **(d) Writes:** none.
- **(e) Defines:** none.
- **(f) Events:** none.

### Phase 3 — Crisis detection + latch (`control_ratio.py:141-159`, `_emit_crisis`:175-208)
- **(a)** If there are no prisoners at all, stop — no crisis is possible. Otherwise compute
  the enforcer-scaled capacity; if prisoners are within it, stop. Otherwise, the FIRST time
  this happens, emit `CONTROL_RATIO_CRISIS` and latch two flags so it never fires again.
- **(b)** `if prisoner_pop == 0: return` (141-142). `max_controllable = enforcer_pop *
  control_capacity` (147). `if prisoner_pop <= max_controllable: return` (150-151). Latch
  guard: `if not persistent.get("_control_crisis_emitted"): emit; persistent[
  "_control_crisis_emitted"] = True; persistent["_control_ratio_crisis_tick"] = tick`
  (154-159). Event-payload arithmetic (`_emit_crisis`): `actual_ratio = prisoner_pop /
  enforcer_pop if enforcer_pop > 0 else float("inf")` (185); `over_capacity_by = prisoner_pop
  - max_controllable` (186).
- **(c) Reads:** Phase-2 fold outputs, `carceral.control_capacity`.
- **(d) Writes:** `context.persistent_data["_control_crisis_emitted"] = True`,
  `["_control_ratio_crisis_tick"] = tick` (non-graph).
- **(e) Defines:** `carceral.control_capacity` (int, default 4, domain `[1,20]` —
  defines.yaml:294, territory.py(defines):272-277).
- **(f) Events:** `EventType.CONTROL_RATIO_CRISIS` — at most once per `Simulation` run (the
  latch has no reset path other than `Simulation.reset()`).

### Phase 4 — Terminal delay gate + bifurcation (`control_ratio.py:161-173`, `_emit_terminal_decision`:210-247)
- **(a)** Once the crisis has been on the books long enough (a configured delay), compute the
  population-weighted average prisoner organization and emit exactly one terminal event:
  "revolution" if that average clears a threshold, else "genocide."
- **(b)** `crisis_tick = persistent.get("_control_ratio_crisis_tick"); if crisis_tick is
  None: return` (162-164 — defensively unreachable, §4). `terminal_delay =
  services.defines.carceral.terminal_decision_delay; if tick < crisis_tick + terminal_delay:
  return` (166-168). `avg_organization = prisoner_org_sum / prisoner_pop if prisoner_pop > 0
  else 0.0` (171 — the `else` branch is likewise defensively unreachable, §4). Outcome
  (`_emit_terminal_decision`): `if avg_organization >= revolution_threshold: outcome =
  "revolution" else "genocide"` (221-232).
- **(c) Reads:** Phase-2/3 outputs, `carceral.terminal_decision_delay`,
  `carceral.revolution_threshold`.
- **(d) Writes:** `context.persistent_data["_terminal_decision_emitted"] = True` (non-graph).
- **(e) Defines:** `carceral.terminal_decision_delay` (int, default 1, domain `[0,52]` ticks —
  defines.yaml:300); `carceral.revolution_threshold` (float, default 0.5, domain `[0.0,1.0]`
  — defines.yaml:297, territory.py(defines):290-295).
- **(f) Events:** `EventType.TERMINAL_DECISION` — exactly once per `Simulation` run, ever
  (the Phase-1 outer guard makes this the system's true terminal state).

**Events emitted by the whole system: 2 distinct `EventType` values** (`CONTROL_RATIO_CRISIS`,
`TERMINAL_DECISION`) — grep-confirmed, no other `EventType` reference anywhere in
`control_ratio.py`. Both are read downstream **only** by AI/narrative/optimization periphery
(§5), never by another engine System — consistent with the CURRENT BSL SURFACE note that
event emissions have no ledger home today (WS1, #502).

**Zero graph writes anywhere in this system** — confirmed by grep (no `update_node`/
`add_node`/`remove_node`). Its only two "outputs" are (i) keys in a Simulation-instance-scoped
Python dict that is not part of `WorldState`, and (ii) ephemeral `EventBus` payloads.

## 3. TYPE INVENTORY

| Attribute | Node type | Python model type | Domain | Category |
|---|---|---|---|---|
| `role` | SOCIAL_CLASS | `SocialRole` (StrEnum, 8 members) | closed set | **Enum discriminant** — read-only here, defensively re-parsed by a bespoke `_get_role` (`control_ratio.py:40-50`) rather than reusing the codebase's own centralized `SocialRole.coerce()` (`models/enums/social.py:45-57`), which exists for exactly this purpose — a duplicated-not-shared helper, port-as-is note. |
| `active` | SOCIAL_CLASS | `bool` | `{T,F}` | boolean flag, model default `True` (`social_class.py:380-383`); read default here **also** `True` (`control_ratio.py:58,78`) — consistent. |
| `population` | SOCIAL_CLASS | `int` | `≥ 0` | integer, model default **1** (`social_class.py:406-410`); read default here is **0** (`control_ratio.py:61,81`) — a divergent fallback, harmless in practice (population is always explicitly seeded on live nodes) but a genuine port-as-is transcription note. |
| `organization` | SOCIAL_CLASS | `Probability` (`Annotated[float, ge=0.0,le=1.0]`, `SnapToGrid`) | `[0,1]` | unit-interval, model default **0.1** (`social_class.py:355-358`); read default here is **0.0** (`control_ratio.py:82`) — same divergent-fallback pattern as `population`. |
| `control_capacity` (define) | — | `int` | `[1,20]` | integer coefficient. |
| `revolution_threshold` (define) | — | `float` | `[0.0,1.0]` | unit-interval coefficient. |
| `control_ratio_delay` (define) | — | `int` | `[0,520]` ticks | integer coefficient. |
| `terminal_decision_delay` (define) | — | `int` | `[0,52]` ticks | integer coefficient. |
| `_terminal_decision_emitted` | `context.persistent_data` | `bool` (implicit — a raw dict key, never a typed field) | `{T, absent}` | **Non-graph Simulation-instance flag.** Not a node/edge attribute, not part of `WorldState` (§5). Lost on save/load. |
| `_control_crisis_emitted` | `context.persistent_data` | `bool` (implicit) | `{T, absent}` | same category |
| `_control_ratio_crisis_tick` | `context.persistent_data` | `int` (implicit) | tick index | same category |
| `_class_decomposition_tick` | `context.persistent_data` | `int \| None` (implicit) | tick index | same category — **written by DecompositionSystem**, read-only here |
| `actual_ratio` (event payload) | — | `float` | `[0,∞) ∪ {+∞}` | **Unbounded real with a genuine `+∞` special case** (`control_ratio.py:185`) — the sharpest single value in this system's surface; ephemeral (event payload only, never graph-stored). |
| `avg_organization` (event payload) | — | `float` | `[0,1]` mathematically (a weighted average of `Probability`-domain inputs), **not independently clamped by this code** | unit-interval, ephemeral. |

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `int`/`float`); grep-confirmed **zero**
`exp`/`log`/`sigmoid`/`pow`/`math.*` calls anywhere in `control_ratio.py` — the same
zero-libm-hazard profile TerritorySystem had. Shapes, in execution order:

1. **Int accumulation (fold sum):** `total += attrs.get("population", 0)`
   (`control_ratio.py:61`) — a role-and-active-filtered fold, iteration order = graph node
   insertion order (deterministic, `query_mixin.py:50`). Maps directly onto BSL's now-landed
   `fold sum`.
2. **Int accumulation (fold sum):** `total_pop += pop` (`:83`) — same shape.
3. **Mixed multiply-then-float-accumulate (fold sum of a per-node product):** `org_sum +=
   pop * org` (`:84`) — `pop` is `int`, `org` is `Probability`(float); the product widens to
   float before accumulating. Same `fold sum` shape as items 1-2, now expressible under the
   landed query lane, but it MIXES an int-typed field and a float-typed field inside one
   product — a concrete instance of the same Currency/Real-lane operator-table question
   Metabolism's D-1 hazard names generically, here for `int × Probability` rather than
   `Currency × Ratio`.
4. **Int multiply:** `enforcer_pop * control_capacity` (`:147`) — int×int, exact, no
   truncation risk.
5. **Int comparison:** `prisoner_pop <= max_controllable` (`:150`).
6. **True division with a bare non-finite fallback literal:** `prisoner_pop / enforcer_pop if
   enforcer_pop > 0 else float("inf")` (`:185`). The `float("inf")` branch is **confirmed
   reachable**, not dead defensive code — `tests/unit/engine/systems/test_control_ratio.py:233
   test_no_enforcers_triggers_immediate_crisis` seeds zero `CARCERAL_ENFORCER` nodes and 100
   `INTERNAL_PROLETARIAT` population, which drives exactly this branch (the test asserts
   `enforcer_population == 0` on the emitted payload; `actual_ratio`/`control_ratio` in that
   same payload evaluate to `float("inf")` even though the test does not assert on them
   directly). BSL's closed `<literal>` grammar (per every landed pack read) has no infinity
   form — this is the single sharpest float-op hazard in the file. It is ephemeral (event
   payload only, never written to the graph), so it does not block a hash-bearing port of the
   state machine itself, but it does block a byte-faithful port of the
   `CONTROL_RATIO_CRISIS` event's own payload content, whenever event content becomes a WS1
   ledger row.
7. **Int subtraction:** `prisoner_pop - max_controllable` (`:186`) — `over_capacity_by`,
   provably positive given the line-150 gate already passed.
8. **Int→Real widening (not a demotion):** `float(control_capacity)` (`:200`) — payload-only
   cast, the reverse direction of Territory's Real→Int demotions; no BSL-side hazard since it
   never touches graph storage.
9. **True division with a provably-dead fallback:** `prisoner_org_sum / prisoner_pop if
   prisoner_pop > 0 else 0.0` (`:171`) — the `else 0.0` branch is unreachable within one
   `step()` call, because `step()` already returned at line 141-142 whenever
   `prisoner_pop == 0`, in the SAME call, before this line is reached. Not a
   producer/consumer defect (ADR183 §5.4's "do not port the defect" class) — ordinary
   defensive coding, transcribable port-as-is as an always-true ternary, or simplifiable at
   the port author's judgment; not a blocker either way.
10. **Threshold comparison:** `avg_organization >= revolution_threshold` (`:221`) — plain
    `>=`, no clamp involved.

**Clamps: NONE.** Zero `min(`/`max(` calls anywhere in `control_ratio.py` (grep-confirmed) —
unlike Territory's two inconsistent `[0,1]` clamp shapes, ControlRatioSystem clamps nothing,
because it writes nothing to the graph. `avg_organization`'s `[0,1]` domain is inherited
entirely from its `Probability`-typed inputs rather than enforced by this code.

**Real→Int demotions: NONE.** No `int(...)` cast anywhere in this file (grep-confirmed) —
contrast with Territory's two `int(x * rate)` truncations.

**Bare non-integer literals:** none of the numeric constants in this file need a BSL
`c`-suffix, because the arithmetic contains no plain numeric literals at all — every number
is either an int/float from a graph/define read or one of the two special-cased `float(...)`
casts above (items 6, 8). The only literal-shaped values in the arithmetic are `float("inf")`
(item 6) and the two dead-branch `0.0`s (item 9).

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 12.0** (`control_ratio.py:98`), confirmed against `_SYSTEM_CLASSES`
  (`simulation_engine.py:328-363`): `... ImperialRentSystem(9.0) → TransportSystem(9.5) →
  DispossessionEventSystem(10.0) → DecompositionSystem(11.0) → ControlRatioSystem(12.0) →
  MetabolismSystem(13.0) → OODASystem(14.0) ...`. Eleven systems run before it every tick.
- **Reads from a same-tick prior system: YES, directly and load-bearingly — unlike
  Territory.** `DecompositionSystem` (position 11.0, the IMMEDIATELY preceding system)
  writes, in the SAME tick, all three of: `SOCIAL_CLASS.active` (`True` on the
  enforcer/internal-proletariat nodes it activates, `False` on the LA node it deactivates —
  `decomposition.py:327-339`), `SOCIAL_CLASS.population` (the freshly-computed 30/70 split —
  `decomposition.py:329,336`), and `context.persistent_data["_class_decomposition_tick"]` /
  `["_decomposition_complete"]` (`decomposition.py:222-223`) — on the very tick decomposition
  first fires, ControlRatioSystem's Phase-2 census counts the population DecompositionSystem
  just wrote, and its Phase-1 gate reads the tick DecompositionSystem just stamped, both
  same-tick.
- `SOCIAL_CLASS.organization`'s only same-tick prior writer is `TerritorySystem` (position
  2.0, `PENAL_COLONY` suppression only — hard-set to `0.0`, `territory.py:378`, per this
  task's own SPECIAL NOTE); on every tick where no `PENAL_COLONY`/`TENANCY` suppression
  fires, the value ControlRatioSystem reads is whatever the PREVIOUS tick's
  `AllegianceSystem` (17.42, cross-tick, not same-tick) or the scenario seed left in place.
- `SOCIAL_CLASS.role` is set once, either at scenario-seed time or by `DecompositionSystem`'s
  `_create_target_entity` (`decomposition.py:225-261`) — never mutated afterward anywhere in
  the engine (confirmed by ADR070's own census: "Role is NEVER mutated at runtime").
- **Writes consumed downstream: effectively none.** ControlRatioSystem writes ZERO graph
  attributes (§2, §4). Its two non-graph outputs:
  - `context.persistent_data` flags — read only by ControlRatioSystem itself, on later ticks
    of the SAME `Simulation` instance (self-referential latch state, not a cross-system
    channel).
  - `EventType.CONTROL_RATIO_CRISIS`/`TERMINAL_DECISION` — grep-confirmed read by **zero**
    other engine Systems (`src/babylon/engine/systems/*.py`); consumed only by
    AI/narrative/optimization periphery: `game/chronicle_adapter.py:292,298` (narrative
    text), `engine/event_builders.py:226,234` (typed-event conversion),
    `models/event_severity.py:304,314` (severity = `"critical"` classification),
    `engine/optimization/objectives.py:76,83` plus
    `engine/optimization/backends/{headless,in_memory}.py` (the headless optimization
    harness's own `terminal_outcome` readout), `intelligence/ai/director.py:120`
    (dashboard). None of these are MATERIAL_BASE/ACTION/CONSEQUENCE Systems — ControlRatio
    is a genuine terminal leaf in the System dependency graph.
- Cross-checked against `src/babylon/sentinels/superstructure/registry.py:95`:
  `control_ratio.py` is a declared MATERIAL_BASE file with **no** superstructure-register
  ownership — it never calls `set_graph_attr`, consistent with the zero-graph-writes finding.
- **Context/service usage with no BSL equivalent: the system's ENTIRE control-flow
  skeleton.** All four `persistent_data` reads/writes (§2 Phases 1, 3, 4) plus
  DecompositionSystem's own three writes to the SAME dict are `context.persistent_data` — the
  Q6 "graph-scope state" gap (`reports/bsl-gap-analysis-2026-08-10.md` §2, "the single most
  pervasive gap in the estate," 22 systems named, INCLUDING ControlRatio and Decomposition by
  name). Verified independently in this session, not merely cited: `simulation_engine.py:
  564-580` shows `TickContext.persistent_data` is seeded from and synced back to a
  `persistent_context: dict` OWNED BY `Simulation` (`engine/simulation/_legacy.py:52,120`) —
  it is `Simulation`-instance Python state, never touching `WorldState`, `graph.graph[...]`,
  or a node/edge attribute (confirmed: `_restore_graph_context`/`_save_graph_context`,
  `simulation_engine.py:436-484`, mirror ONLY `_base_year`/`_tick_dynamics`/
  `_national_financial` — none of the six carceral-phase flags). **This means the frozen
  Python reference itself would lose all four of ControlRatioSystem's latches (and
  Decomposition's three) on a save/load round-trip** — a genuine pre-existing defect in the
  reference the port inherits knowledge of. Port-as-is law: transcribe the BSL-storage gap
  honestly; do not silently "fix" a durability the Python reference never had.
- **Vocabulary gap not caught by `check:vocabulary`** (that sentinel covers `_node_type` and
  attribute-shape, not enum member VALUES): `SocialRole.LUMPENPROLETARIAT` is one of
  ControlRatioSystem's two `_PRISONER_ROLES` members (`control_ratio.py:32-37`), but a grep
  across `src/babylon/engine/scenarios/_legacy.py` (the sole canonical-scenario node factory,
  1270 lines, fully checked) finds **no** `role=SocialRole.LUMPENPROLETARIAT` node creation
  anywhere. The role is real vocabulary (a bracket key in `wealth_distribution.py:67`,
  `epistemic_horizon.py:75`, `struggle.py:53,489`, `hydration/reference.py`'s share tables)
  but has no node-producer anywhere in the engine — every prisoner ControlRatioSystem has
  ever counted, in every scenario in this repository, has role `INTERNAL_PROLETARIAT`.
- **DORMANCY on canonical scenarios — provable, not just plausible.**
  `tools/regression_test.py:81` sets `DEFAULT_MAX_TICKS: Final[int] = 52`, and
  `tools/regression_scenarios.py`'s `SCENARIOS` dict (2925 lines, fully grepped) carries no
  `carceral.*` `defines_overrides` and no per-scenario tick-count override anywhere.
  ControlRatioSystem's own gating defines require `tick >= decomposition_tick +
  control_ratio_delay` (`control_ratio_delay` default 52) just to reach the capacity check,
  and `decomposition_tick >= 1` at the very earliest (decomposition can only fire from
  INSIDE the tick loop, which starts at tick 1 — `tools/regression_test.py:1054`,
  `for tick in range(1, max_ticks + 1)`) — so the earliest the capacity check could possibly
  run is tick 53, one past the 52-tick `qa:regression` horizon. **ControlRatioSystem is
  provably 100% dormant on every `qa:regression` canonical scenario, past Phase 1's readiness
  gate — Phase 2's census never fires either.** The scenario factory DOES seed the needed
  node/role structure (`CARCERAL_ENFORCER`/`INTERNAL_PROLETARIAT`, dormant —
  `engine/scenarios/_legacy.py:372-404`), unlike Territory's missing `ADJACENCY` edges, so
  this is a HORIZON gap, not a STRUCTURE gap — a longer `qa:regression` run (or a
  `carceral.*` override shortening the delays) would exercise it. The only place in the
  repository that actually drives ControlRatioSystem through all four phases end-to-end is
  `tests/scenarios/test_carceral_equilibrium.py` (`MAX_TICKS = 5200`, 100 simulated years) —
  see §7.

## 6. BLOCKER ASSESSMENT

Adjudicated against the CURRENT BSL surface (Query-lane Slice 1 / ADR197 landed
2026-08-11; enum fields ADR195/196 landed; `<bind-src>` still closed at
`:field`/`:const`/`:metric`/`:tick`; events have no ledger home).

| Computation | Verdict | Detail |
|---|---|---|
| Phase 1 — readiness gate (`context.persistent_data` reads, `control_ratio.py:119-134`) | **BLOCKED — Q6 (graph-scope state)** | `bsl-language.rst` §2.5 closes `<bind-src>` at exactly `:field`/`:const`/`:metric`/`:tick` — none can name `context.persistent_data`, confirmed unchanged from the 2026-08-10 survey (no landing announced in the CURRENT BSL SURFACE). **A concrete, verified escape route exists**, not merely cited: `babylon-bsl/src/tick.rs:159-182`'s `subject_type_of` derives a rule's subject purely from its `:field` bindings' shared namespace and **errors** on zero bindings ("the rule declares no :field binding... slice 1 runs rules over a population, not over the graph as a whole") — so a rule scoped to ONE singleton carrier node (its `:field` bindings all in that node's own namespace, per Q6's own recommended route (a)) satisfies subject-type derivation AND fires exactly once per tick "for free," without ever needing Q12's `(domain :graph)` to be consumed at execution time — confirmed it currently ISN'T (`domain.rs` resolves `RuleDomain::Graph` at LOAD time only; `run_tick`, `tick.rs:524`, never reads `loaded.domain`). D-record candidate: mint a new singleton NodeType (Q6's own text calls this "amendment territory under §3.6" — not softened here) holding `decomposition-tick`/`control-crisis-emitted`/`control-ratio-crisis-tick`/`terminal-decision-emitted` as declared `int`/`bool` fields. |
| Phase 2 — population/organization census (`control_ratio.py:53-85`) | **PORTABLE WITH D-RECORD** | D-record: declare `social-class/role enum SocialRole` (new — no landed pack declares a `SocialRole`-shaped enum yet, though `social-class/organization int extensive` is already declared once, `query-lane-e2e.bscn:57`, and enum declare+`=`-compare has a working precedent, `organization-foundation.bscn:45,57,61` + `organization.bsl:29`). Mechanically the fold itself (a filtered `fold sum` over a typed node population with an enum-equality `:when` guard) is **portable now** under the landed Query-lane Slice 1 + enum fields — this corrects the 2026-08-10 gap report's `(blocked)` tag (§1 row 60), which predates both landings (ADR197 merged 2026-08-11, per this repo's own commit log at HEAD: PR #521). |
| Phase 3 — crisis threshold + `CONTROL_RATIO_CRISIS` emit (`control_ratio.py:141-159`, `_emit_crisis`:175-208) | **PORTABLE WITH D-RECORD**, contingent on Phase 1's singleton landing | The arithmetic itself (int multiply, int compare, int subtract) is trivial. The `float("inf")` special-value payload field (§4 item 6) is a SEPARATE, smaller D-record (no BSL literal for infinity); currently moot for a hash-bearing port of the state machine (events aren't hashed today) but real for a future byte-faithful event-content port. |
| Phase 4 — terminal delay gate + bifurcation (`control_ratio.py:161-173`, `_emit_terminal_decision`:210-247) | **PORTABLE WITH D-RECORD**, same Phase-1 contingency | The `>=` threshold comparison and the two-branch outcome string are trivial once `avg_organization` is computable (Phase 2) and the crisis tick is readable (Phase 1). |
| The whole system, structurally (not a single computation) | **NOT PORTABLE IN ISOLATION — Class D** | `bsl-gap-analysis-2026-08-10.md` (~line 662): "Reachable only downstream of another system's emission. Port them [Decomposition, ControlRatio] together with their producer, because the port also replaces the event-history read with a producer-written field (Q16). Porting them apart would leave a rule reading a field nothing writes." DecompositionSystem is ControlRatio's sole producer of the gating tick (§5) — the Phase-1 D-record must be co-designed with, or land inside, Decomposition's own port. |
| `SocialRole`-based role reads throughout | **RESERVED-LINE ADJACENT — port-as-is, do not attempt the cutover** | ADR070/Program 19 rules ControlRatioSystem's revolution-vs-genocide branch the explicit LAST system in the emergent-class-partition adjudication cutover — "no exception... only after low flip-count evidence, with a dedicated high-effort review." A BSL port that transcribes the current `SocialRole`-keyed reads verbatim (the Phase-2 D-record) is consistent with ADR070's own "slots-as-positions" ruling (role is real, persistent seed vocabulary, not something a port should derive); a port that instead tried to swap ControlRatio onto the derived class-cell partition would pre-empt a Director-gated, explicitly-deferred architectural decision — describe, never propose. |

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_control_ratio.py` | 756 | **Primary conformance-oracle candidate.** 21 unit tests across 3 classes (`TestControlRatioSystem`, `TestTerminalDecision`, `TestControlRatioMutationKillers`): crisis threshold, narrative-hint payload shape, inactive-entity exclusion, the zero-enforcer edge case (the `float("inf")` branch, §4), revolution/genocide bifurcation, delay-gate boundaries, one-shot latch behavior, exact-organization weighted-average arithmetic. The "MutationKillers" class exists specifically to pin `<=` vs `<` boundary behavior — exactly the port's own conformance target. |
| `tests/unit/engine/laws/test_law_control_ratio.py` | 270 | **Property-based invariant contracts** — explicitly written with a future port in mind (its own docstring: "P27 Phase-0 coverage backfill, Task 11... Read end-to-end before writing: `control_ratio.py`"). Four laws: crisis-threshold clamp (`TestCrisisThresholdClampLaw`), no-op-on-zero-prisoners (`TestInactivityOnEmptyPrisonersLaw`), avg-organization monotonicity in one input (`TestAvgOrganizationMonotonicityLaw`), idempotent latch — crisis and terminal each fire at most once (`TestIdempotentLatchLaw`). The sharpest behavioral-contract candidates in the estate for this system — laws survive a rewrite the way bit-exact goldens don't. |
| `tests/integration/mechanics/test_control_ratio_crisis.py` | 373 | Integration-level: 10 tests across 3 classes (`TestControlRatioCrisis`, `TestTerminalDecision`, `TestControlRatioEdgeCases`) — largely a broader-fixture re-statement of the unit suite (crisis/no-crisis, revolution/genocide, exact-threshold, delay respect, one-shot emission), narrative-adjacent in places (asserts `narrative_hint` presence). |
| `tests/scenarios/test_carceral_equilibrium.py` | 436 | **The only live end-to-end conformance oracle in the estate for this system.** `MAX_TICKS = 5200` (100 simulated years, `weeks_per_year`-scaled) — runs `create_imperial_circuit_state()` through `Simulation.step()` in a loop, asserting the FULL five-phase order (metabolic rift opens → `SUPERWAGE_CRISIS` → `CLASS_DECOMPOSITION` → `CONTROL_RATIO_CRISIS` → `TERMINAL_DECISION`) and the default "genocide" outcome (no player organization). This is the only place in the repository where ControlRatioSystem's Phase 2/3/4 computations (§2) actually execute against anything resembling live simulation state — `qa:regression`'s 52-tick horizon never reaches them (§5). |
| `tests/unit/engine/test_system_order.py` | 300 | Schema-level: pins ControlRatioSystem's name/position (12.0) in `_SYSTEM_CLASSES` ordering — confirms tick position, not behavior. |
| `tests/unit/models/test_event_severity.py`, `tests/unit/models/test_enums.py` | (grepped, not read in full) | Schema-level: `EventType.CONTROL_RATIO_CRISIS`/`.TERMINAL_DECISION` membership + severity `"critical"` classification — pins the enum surface, not the system's math. |
| `tests/integration/mechanics/test_class_decomposition.py`, `tests/unit/engine/laws/test_law_decomposition_system.py`, `tests/unit/engine/systems/test_decomposition_enforcer_creation.py`, `tests/unit/engine/systems/test_la_decomposition.py` | (not read; located by `SocialRole`/`ControlRatio`-adjacent grep) | DecompositionSystem's own test estate — adjacent, not primary, but relevant given the Class-D coupling (§6): the ControlRatio port's conformance fixture will likely need to seed decomposition-adjacent state the way these tests do. |

**No dedicated `qa:regression`/dense-golden coverage.** Given §5's dormancy finding,
`tools/regression_test.py::graph_content_hash`'s byte-identical gate hashes a graph state
that never differs by ControlRatioSystem's action on any canonical scenario — the gate
provides **zero** regression protection for this system today, unlike Territory (whose Phase
1 heat dynamics ARE live on every canonical scenario). A port's conformance fixture must be
hand-built (a short-delay `carceral.*` override, or a hand-seeded `.bscn` with pre-activated
enforcer/prisoner populations), exactly as Metabolism's and Territory's own Phases 2-4 needed.

---

## Adjudication (2026-08-12)

Adjudicated against the dev tree at `9324482f`. Three corrections, four confirmations. This
inventory did the most Rust-side verification in the batch and its escape route survives intact;
the corrections are all on Phase 2's "portable now" row and on the dormancy framing.

1. **CONFIRMATION — the singleton-carrier escape route is real, verified independently, and is
   the ONLY viable one on current dev.** `subject_type_of` (`rust/crates/babylon-bsl/src/tick.rs:159-182`)
   derives the subject purely from the shared namespace of the rule's `:field` bindings and errors
   on zero bindings with exactly the quoted text — *"the rule declares no :field binding, so it
   names no subject type — slice 1 runs rules over a population, not over the graph as a whole"*
   — and errors on disagreement with *"a field is a field OF self's node type"*. A rule whose
   `:field` bindings all sit in one one-member carrier namespace therefore fires exactly once per
   tick without Q12. And Q12 is confirmed unconsumed at execution: a grep for `.domain` /
   `loaded.domain` across `tick.rs` returns **nothing**, while `RuleDomain::Graph` resolves at
   load only (`domain.rs:214`).
   **Adding the reason the alternative is closed** (the inventory reached the right route without
   recording why the spec's own one is unavailable): D40's accessor `the` sits in
   `UNSERVED_EXPRESSION_HEADS` tagged `"slice 2"` (`evaluator.rs:505`); its singleton guard exists
   only as a LOAD-time check — `E-LOAD-043`, *"(the {row}) needs a declared :ceiling of exactly 1"*
   (`manifest.rs:51-53, 100-104`) — and **no landed content declares a `manifest` at all** (zero
   `manifest` hits across `rust/crates/babylon-tick/content/`). The `DecompositionSystem`
   inventory, which must co-design this carrier, proposes `the` explicitly and has been corrected
   to this route citing this row.

2. **CONFIRMATION — the zero-graph-writes and `float("inf")` findings hold at the byte.** A grep
   for `update_node|add_node|remove_node|set_graph_attr|update_edge` across `control_ratio.py`
   returns nothing. `actual_ratio = prisoner_pop / enforcer_pop if enforcer_pop > 0 else
   float("inf")` at `:185`; `float(control_capacity)` at `:200`; all six `persistent_data`
   touches at `:124, 128, 154, 158-159, 162, 173`. The Q6 characterisation — that this system's
   entire control-flow skeleton is non-graph state the frozen reference itself loses on save/load
   — is correct and correctly refused a silent "fix."

3. **CONFIRMATION — the stale-comment catch is right, and it is a genuine correction to the
   estate.** `bool` IS a valid `deffield` type today: `"bool" => Ok(BslType::Bool)`
   (`declarations.rs:649`), and the in-repo comments asserting otherwise are stale
   (`content/scenarios/vitality-conformance.bscn:20`,
   `vitality-lifecycle-combined-conformance.bscn:34` — *"BSL has Bool (§3.1) but `deffield` has no
   bool"*). See correction 5 for what that landing does and does not buy this system.

4. **CORRECTION — Phase 2's D-record ("declare `social-class/role enum SocialRole`") would BREAK
   the fold it is filed to enable.** The census reads `role` off an **arbitrary iterated node** —
   `_get_role(attrs)` inside a `query_nodes(SOCIAL_CLASS)` loop at `control_ratio.py:60` and
   `:84` — so the BSL shape is `field-of it social-class/role` inside a `fold`, not a `:field`
   binding. `field-of` is **REFUSED** for `:enum-type`-declared fields (D102), enforced at LOAD by
   `check_no_field_of_on_enum_field` (`rust/crates/babylon-bsl/src/typecheck.rs:246-280`, wired at
   `rule_pipeline.rs:297`) with the message *"… not extended to enum-declared fields (§2.13,
   D102)"*; spec at `docs/reference/bsl-language.rst:2274-2284`, register row at `:5681-5693`.
   `field_of_node` itself (`evaluator.rs:1274-1292`) is the only accessor that reads an arbitrary
   queried element's node field, so there is no second path. The workable declaration is the
   **int-ordinal encoding** (`content/rules/lifecycle.bsl`'s landed convention) — precisely what
   this batch's `DecompositionSystem` inventory concludes at length for the identical read shape.
   The row's other half is correct and important: the fold itself IS portable now under Slice 1,
   and the 2026-08-10 survey's `(blocked)` tag is genuinely stale. Re-file as *"PORTABLE WITH
   D-RECORD — under Slice 1 + **int-ordinal** role, NOT `deffield … enum`."*
   Two smaller precision notes on the same row: `_PRISONER_ROLES` (`control_ratio.py:32-37`,
   used at `:84`) is a two-member SET test, so the predicate is a disjunction of two comparisons,
   not one enum equality; and that predicate lives in the fold body over `it`, not in a `:when`
   guard — the guard is per-subject, and under correction 1's route this rule's subject is the
   carrier node, not the counted class.

5. **CORRECTION — the `active` filter (`control_ratio.py:58, 78`) is a second, unnamed D-record
   on the same "portable now" row, and §1's `bool`-is-landed finding cannot carry the weight §6
   puts on it.** Even with `bool` a legal `deffield` type, a boolean is never READ as a
   `Value::Bool`: `bind_field_value` returns `Value::Real(stored)` for every non-enum declared
   type — only the `BslType::Enum` branch renders anything else (`tick.rs:312-327`) — and
   `field_of_node` returns `Ok(Value::Real(value))` unconditionally (`evaluator.rs:1281-1291`).
   Downstream, `as_bool` refuses a Real where a `<cond>` is required (`evaluator.rs:1315-1320`)
   and `apply_equality` refuses it against a `#t`/`#f` literal — *"equality is defined within one
   lane only"* (`evaluator.rs:1620-1628`); on the write side `numeric_write_value` refuses a
   `Value::Bool` outright (`structural_verbs.rs:1231-1233`). The filter is expressible only as
   `(= (field-of it social-class/active) 1)` over a 0/1-encoded `int` field. Declaring `bool` buys
   a load-time range check and nothing at evaluation.

6. **CORRECTION — §5's "this is a HORIZON gap, not a STRUCTURE gap — a longer `qa:regression` run
   (or a `carceral.*` override shortening the delays) would exercise it" is unsupported, and its
   parenthetical is falsified by the estate's own ledger.** `DEFAULT_MAX_TICKS: Final[int] = 52`
   is confirmed (`tools/regression_test.py:81`), and the 52-vs-53 arithmetic is correct as far as
   it goes — but it is not what gates this system. Phase 1 returns on
   `persistent.get("_class_decomposition_tick") is None` (`control_ratio.py:128-130`), and the
   `DecompositionSystem` coverage-gap row states that key is never written because *"SUPERWAGE_CRISIS
   never fires (neither ImperialRentSystem's pool-exhaustion path nor this system's own
   approaching-death early-warning path) **within 150 ticks** in any of the five scenarios"*
   (`tools/regression_scenarios.py:2817-2823`). Both `carceral.control_ratio_delay` and
   `carceral.decomposition_delay` (`defines.yaml:298-299`) are measured **from an event that never
   fires**, so shortening either cannot exercise this system at all. The ledger's own remediation
   is calibration, not horizon — *"a longer-horizon **or more austerity-calibrated** scenario that
   actually exhausts the imperial rent pool or starves the labor aristocracy"* — and the only live
   driver is a purpose-built 5200-tick fixture on a DIFFERENT factory
   (`tests/scenarios/conftest.py`'s `create_imperial_circuit_state()`, per the Decomposition
   inventory's §5). The dormancy conclusion is unaffected and if anything stronger; the framing
   would mislead a port author sizing the conformance fixture, which is the one decision this
   section exists to inform.

7. **CONFIRMATION — position, channels and the reserved-line flag.** Tick position 12.0
   (`control_ratio.py:98`), immediately after `DecompositionSystem` (11.0, `decomposition.py:102`)
   and before `MetabolismSystem` (13.0, `metabolism.py:50`), against `_SYSTEM_CLASSES`
   (`simulation_engine.py:328-360`), which `_DEFAULT_SYSTEMS` derives by sorting on `position`
   (`:376-378`). The Class-D coupling is exactly one key wide and verified from both ends:
   `decomposition.py:223` writes `_class_decomposition_tick`, `control_ratio.py:128` reads it.
   The `LUMPENPROLETARIAT`-has-no-producer finding holds — a grep across `src/babylon/` finds the
   member only as a defines key or a bracket key (`wealth_distribution.py:67`, `struggle.py:53,489`,
   `epistemic_horizon.py:75`, `community.py:51`, `control_ratio.py:35`) and never as a `role=`
   node construction. **The RESERVED-LINE flag is correctly raised and correctly handled**:
   ADR070/Program 19 rules this system's revolution-vs-genocide branch the explicitly-LAST system
   in the emergent-class-partition cutover, and "port-as-is, describe never propose" is the right
   disposition. Worth carrying into the joint Class-D train: `DecompositionSystem` raises no
   reserved-line surface of its own, so this flag travels with the pair via this half alone.

**FINAL VERDICT: BLOCKED on Q6 (graph-scope state — `context.persistent_data` has no BSL
`<bind-src>` or write-verb), with the singleton-carrier D-record escape route CONFIRMED as the
only one available on current dev, and the joint Class-D port with `DecompositionSystem`
CONFIRMED from both sides — UPHELD.** One sub-verdict moves: Phase 2's census is **not** portable
now "under the landed Query-lane Slice 1 + enum fields" — it is portable now under Slice 1 plus
the **int-ordinal** role encoding (D102 refuses `field-of` on an enum-declared field, correction
4) plus a boolean 0/1 encoding for the `active` filter (correction 5). The correction to the
2026-08-10 survey's stale `(blocked)` tag stands and is the right call; it just lands one
declaration to the left of where the inventory puts it. Dormancy is total but
**calibration-caused, not horizon-caused** (correction 6) — a conformance fixture must recalibrate
the trigger, not merely run longer.

**INADEQUATE-COVERAGE NOTE (narrow).** §1's Rust-side list (`tick.rs`, `domain.rs`,
`declarations.rs`, two `.bscn` files) is the strongest in the batch and is why the escape route is
right. The two files it omits are exactly the two that own Phase 2's grade: `typecheck.rs:246-280`
+ `rule_pipeline.rs:297` (the D102 gate) and `evaluator.rs:1274-1292, 1315-1320, 1594-1632`
(`field_of_node`'s unconditional `Value::Real`, `as_bool`, `apply_equality`'s one-lane rule). A
re-read must add both before re-grading any row that reads a field off an iterated element rather
than off `self` — which, in this system, is the entire census.
