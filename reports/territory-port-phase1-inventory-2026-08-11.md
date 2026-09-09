# Territory Port — Phase-1 Inventory and Adjudicated Verdict (2026-08-11)

**Status:** Phase-1 gate CLOSED with verdict **DEFER — query-evaluation train first**.
**Supersedes in part:** the 2026-08-11 morning STOP assessment (its two named blockers —
the Currency scale op and int-only seeding — are both resolved by PR #500 and PR #505;
this inventory finds the deeper gap those blockers masked).

**UPDATE (2026-08-11, later the same day) — the blocking train has landed.** The
query-evaluation train this verdict's "Why DEFER beats an honest-partial pack" section
named as the unblock is `docs/superpowers/plans/2026-08-11-bsl-query-evaluation-plan.md`
(P27 Phase 2 Slice 1), and it now stands COMPLETE: sixteen tasks across five PR groups
(⟨PR 1⟩ #509, ⟨PR 2⟩ #514, ⟨PR 3⟩ #519, ⟨PR 4⟩ #520, ⟨PR 5⟩ — branch
`feat/bsl-query-eval-group5`, not yet merged to `dev` at the time of this update).
⟨PR 5⟩'s Task 15 (`rust/crates/babylon-tick/tests/query_lane_e2e.rs`) is this
inventory's own §6 blocker table, proved: `select-max` with the §2.7 language-level
tiebreak feeding `update-node` against a computed reference (blocker rows 1-2 —
`_find_sink_node`/the population transfer, `territory.py:139-194,259-267`); a
pull-side `fold sum` over `neighbors` reading PRE-tick state (blocker row 3 —
`_process_spillover`, `territory.py:269-316`, and the vector this inventory's own §5
named "structurally dormant on every canonical scenario"); and `for-each` writing a
`TENANCY`-incident subject set (blocker row 4 — `_suppress_organization`,
`territory.py:353-378`) — all through the REAL `run_once_into` production seam, on a
hand-built fixture, exactly as this inventory's §5 anticipated ("A port's conformance
fixtures will need to be hand-built … not harvested from the canonical scenarios").
The `exists` guard this verdict's parent plan derived (the "two further
requirements … not in that table" note) proves out alongside it: an empty `ADJACENCY`
neighbourhood takes the fallback branch, never `E-EVAL-021`. **Full record:**
`ai/decisions/ADR197_bsl_query_evaluation_slice1_handoff.yaml`. **Prior art
(§ "Prior art to reconcile," above): DISCHARGED** — PR #464 now stands closed, its
verdict (SUPERSEDE, three ideas harvested) posted citing the plan by path.

**What this update does NOT do.** No Territory content ships in ⟨PR 5⟩ or this update
— the fixture is synthetic, not the frozen system transcribed. Every item this
inventory filed OUT of scope for the eventual pack stays exactly as filed: the
enum-field-storage gap (finding 4, now also register row D111/query-evaluation-plan
Q9 in `docs/reference/bsl-language.rst`, restated named rather than resolved), the
two-clamp inconsistency (Phase-1 `_write_clamped [0,1]` vs. Phase-3's upper-only
`min(1.0, …)` — the port must transcribe both shapes faithfully, port-as-is, and
Task 15's own spillover vector deliberately never needed the clamp, choosing seed
heats that stay under it rather than expressing it), the `rent_spike_multiplier`
scaled-Int workaround, and the #502 WS1 cross-system-channel ledger items. The
Territory port train itself — an actual Director dossier, a real `.bscn`/`.bsl`
transcription of `territory.py`, its own conformance oracle — has not started; this
update closes the BLOCKING dependency, not the port.

**UPDATE (2026-08-12) — the port itself has landed.** PR B of the Territory port plan
(`docs/superpowers/plans/2026-08-12-territory-port-plan.md`, Tasks 3-8) ships the actual
transcription this report's first UPDATE stopped short of: `territory-conformance.bscn`
(twelve territories, three social classes, seven edges — every conformance case named
in this report's §5/§6 in one hand-built world) plus its frozen-engine mirror
(`territory_conformance.py`, the STRUCTURE oracle, ADR183), and `territory.bsl`'s five
rules — `p1-heat-dynamics`, `p2-eviction-pipeline`, `p3-spillover`, `p4-camp-decay`,
`p4-penal-suppression` — byte-ordered at ONE anchor position, deliberately relying on
D116's cross-rule divergence rather than fighting it (register row D120). All four §6
BLOCKED rows this report's own table named are now DISCHARGED, not merely unblocked:
sink selection with the §2.7 tiebreak (D124 restates the frozen-vs-language tiebreak
divergence this report's finding 4 already flagged, unresolved by design — both stand,
independently correct within their own systems), the population transfer against a
computed reference, the pull-side spillover fold (register row D128 records the
measured, not bit-guaranteed, agreement with the frozen per-edge accumulation), and
`for-each`-driven `PENAL_COLONY` suppression — `territory_conformance.rs`'s 21-test
suite plus a fifth `tick_goldens.rs` pin are the evidence.

**The enum-field-storage gap (finding 4) — DISCHARGED, exactly as this report's
"Enum discriminant flag" section named it should be, by D102/ADR195/ADR196, not by
either workaround this report proposed.** The port needed neither the `bool`-per-value
nor the `int`-ordinal encoding this report floated: the Organization foundation
train's `enum` `<type-name>` row (D101, ADR195) and vocabulary ceremony (ADR196) landed
first, and the Territory port train's own D102 discharge (PR A, `field-of` over an
enum-declared field) closed the LAST piece — `_find_sink_node` reading a NEIGHBOUR's
`territory_type` needed exactly the accessor D102 had deferred. `territory/profile` and
`territory/territory-type` declare `enum OperationalProfile`/`enum TerritoryType`
directly, in the frozen Python declaration order (hash-bearing, ADR195) — D111/Q9
(this report's own register pointer) is now the historical record of a gap that closed
before the port needed a workaround, not a live decision the port had to make.

**What this update does NOT resolve — recorded, not fixed, same posture as the query-
evaluation update above.** The two-clamp inconsistency (D125), the scaled-Int rent lane
(D122), the directed-vs-any sink/spillover walk asymmetry (D123), the summation/apply
float-order divergence (D128), and the `displacement_mode` -> `EXTRACTION` WS1 ledger
item (D129) are all transcribed faithfully and D-recorded in `territory.bsl`'s own
header and the register (`docs/reference/bsl-language.rst`, rows D120-D131) — never
silently repaired, per the Director's port-as-is ruling. Full record:
`ai/decisions/ADR199_territory_port_handoff.yaml`.

## Adjudicated verdict

The frozen `TerritorySystem` (378 lines, `src/babylon/engine/systems/territory.py`) was
inventoried line-by-line by a read-only agent (report reproduced in full below), then the
blocker table was adjudicated against the **current dev tree** (`7d60c635`), because the
agent cited two stale sources (the pre-#480 gap report and `vitality.bsl`'s pre-#489
header). Three corrections and one confirmation:

1. **CORRECTION — `floor` is LANDED** (PR #489, ADR188 Row 2): `declarations.rs:110`
   lists `DECLARABLE_INTRINSICS: ["exp", "log", "floor"]`. The agent's "Real→Int demotion
   BLOCKED" rows (Phase-2 displacement, Phase-4 camp decay) are wrong on current dev. Both
   sites compute `int(x)` for provably non-negative `x`, where truncation ≡ floor — both
   are expressible today with a declared `floor` intrinsic.
2. **CORRECTION of the memory's landing claim, the other direction — the graph-seam heads
   are spec'd and type-checked but NOT evaluated.** The R9 step-3 merge (#480) landed
   grammar, scope, typecheck and bound-checking for the 27 `GRAPH_SEAM_HEADS`
   (`evaluator.rs:364-392`: `fold`, `neighbors`, `select-max`/`select-min`, `field-of`,
   `edge-between`, `for-each`, `exists`, `forall`, `the`, …), but the expression core
   **refuses every one of them in expression position** ("Task 16 / the Phase-2 query
   evaluator", `evaluator.rs:421-430`), and `tick.rs` evaluates rule guards through
   exactly this core (the single `evaluate` call, `tick.rs:404`). To be precise about
   the split: the *effect verbs* among the 27 (`update-node`, `add-node`, `emit`, …)
   ARE served — in effect position, by `structural_verbs.rs` — and every landed pack
   uses them; what no pack on dev uses, because nothing evaluates them, is the
   query/fold/selection/accessor heads. Effect execution is not blocked by Task 16;
   query evaluation inside expressions is. The earlier
   "Q4/Q5/Q8/Q9 landed" reading was true of the *spec surface*, not the evaluator.
3. **CONFIRMATION — the #500 scale op does NOT serve `rent_level × rent_spike_multiplier`.**
   `rent_level` is `:field`-sourced, so it evaluates as `Value::Real`, never
   `Value::Currency` — the same post-#500 discovery Metabolism's D-1 recorded for
   `entropy_factor`. The port would use the same bare-scaled-Int workaround under the same
   ADR183 declared-deviation class, and the same #502 WS3 retirement (the Real-lane
   declared-domain op) applies.
4. **NEW FINDING (agent's, verified) — enum field storage has no BSL representation.**
   `deffield`'s closed type vocabulary has no `enum` row and `Enum<T>` is
   typechecker-only (bsl-language.rst §3.1). `profile` (2-valued) and `territory_type`
   (5-valued) need a content-modeling workaround (one `bool` / int-ordinal) with its own
   D-record. No existing Q-item names this gap.

## Why DEFER beats an honest-partial pack

What is portable **today** is exactly: Phase-1 heat dynamics and Phase-4 camp decay.
Phase-2 eviction cannot land its latch/rent-spike without displacement (that would
*diverge from* the frozen engine, not partially reproduce it — the pipeline is one
behavior), and displacement routing, spillover, and penal-colony suppression all need
graph-query evaluation (typed `neighbors`, `fold`, `select-max`, incidence accessors) plus
cross-node effect targeting. That surviving subset is materially the "sliver-only port
(camp decay + heat)" already **rejected as silent scope shrink** in the morning STOP
session. Landing it as a pack would spend a full port train on the sliver anyway.

Meanwhile the canonical estate exercises none of the blocked surface: no canonical
scenario seeds `HIGH_PROFILE`/`PENAL_COLONY`/`CONCENTRATION_CAMP` territories (declared
coverage gap, `tools/regression_scenarios.py:2678-2683`) and the canonical factories emit
**no ADJACENCY edges** by III.11 honesty (no real county-adjacency reference source at
seeding time). Conformance oracles for Phases 2-4 must be hand-built `.bscn` fixtures in
either sequencing — nothing is lost by deferring.

**The unblock is one train, and it is already the ruled path.** R8/R9 rule that BSL
expansion precedes system ports. The query-evaluation train (serving the graph-seam heads
at tick layer against `GraphSubstrate`, fuel-metered, deterministic iteration order, per
the already-normative bsl-language.rst chapters) unblocks Territory COMPLETE — and it
gates most of the remaining Material Base ports too (Production @3.0 aggregates over
classes; Solidarity @8.0 is edge-centric). Territory then ports once, completely, per the
no-MVP standing feedback.

**Prior art to reconcile at that train's design gate:** open PR #464
(`feat/p2-slice2-query-trait`, pre-Bevy P27 Phase-2 Slice-2 groundwork, 5 commits of its
own and 229 commits behind dev) built the babylon-graph-side §2.6 query accessors. Its disposition — rebase,
harvest, or supersede — is that train's first decision, not decided here.

## Items filed out of this inventory

- **#502 WS1 (cross-system channels):** `TickContext.displacement_mode` is a
  harness-only override — no production path sets it, and the four AUTO-mode threshold
  defines (`elimination_rent_threshold` etc., defines.yaml:244-247) are read by nothing.
  Provably `EXTRACTION` on every live run; the port declares it `:const`
  (Metabolism-D-2-style "provably uniform") and the override machinery goes to the WS1
  ledger. The dead AUTO-mode defines are a WS4 adjudication row (bless as reserved, or
  retire).
- **Port-time D-record candidates (recorded for the eventual pack):** enum→bool/ordinal
  encoding for `profile`/`territory_type`; the two-clamp inconsistency (Phase-1 heat uses
  `_write_clamped` `[0,1]`, Phase-3 spillover hand-writes an upper-only
  `min(1.0, …)` — territory.py:137 vs :315) — the pack must transcribe both shapes
  faithfully, not unify them (port-as-is law); `rent_spike_multiplier` scaled-Int
  workaround (D-1 class).
- **Channel facts for the post-port refactor ledger:** `TERRITORY.population` is read by
  13 downstream systems (Territory's single most load-bearing output);
  `SOCIAL_CLASS.organization` hard-set is consumed by ControlRatio/Survival/Struggle/
  Allegiance; `heat`/`rent_level`/`under_eviction` are read by **no** other system
  (terminal/observational outputs).

---

# Appendix: the read-only inventory (agent report, verbatim)

> Blocker-table rows in §6 below are superseded where they conflict with the
> adjudication above (floor; the Q-item evaluation status). Everything else stands.

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/territory.py` | 378 | **The target.** `TerritorySystem`, all four phases (heat / eviction / spillover / necropolitics). Self-contained — no calls into `formulas/` or `domain/` (grep-confirmed zero hits for `EventType`, and the only imports are `kernel.tick_partition`, `models.enums`, `kernel.system_base`, `kernel.system_protocol` — territory.py:19-37). |
| `src/babylon/config/defines/territory.py` | 624 (whole module; `TerritoryDefines` is lines 12-144) | Coefficient source — `TerritoryDefines` Pydantic model. |
| `src/babylon/data/defines.yaml` | (territory block: lines 234-252) | Player-editable coefficient values, `territory:` section. |
| `src/babylon/models/entities/territory.py` | 352 | `Territory` Pydantic entity — field types/domains for every attribute TerritorySystem reads/writes. |
| `src/babylon/models/enums/territory.py` | 184 | `OperationalProfile`, `TerritoryType`, `DisplacementPriorityMode` enums. |
| `src/babylon/models/enums/topology.py` | 253 | `NodeType.TERRITORY`, `EdgeType.ADJACENCY`/`EdgeType.TENANCY`. |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase._write_clamped` (territory.py:137 uses it), `._publish`, `._get_persistent_data`. |
| `src/babylon/kernel/graph_protocol.py` | 494 | `GraphProtocol` — `query_nodes`/`query_edges`/`get_node`/`update_node` signatures. |
| `src/babylon/topology/graph.py` | (relevant: `update_node` at 660-670) | Concrete `BabylonGraph.update_node` — plain dict merge, **no quantization at tick time** (see §3). |
| `src/babylon/kernel/math.py` | 61 | `quantize()` — the `SnapToGrid` 1e-5 grid rounder used by Pydantic types, **not** invoked mid-tick. |
| `src/babylon/models/types.py` | 337 | `Currency`/`Intensity`/`Probability`/`Coefficient` — all `Annotated[float, ...]`, i.e. the Python "Currency" is a **plain unbounded-above `[0,∞)` float**, unrelated to BSL's `i128` `Currency`. |
| `src/babylon/engine/context.py` | 113 | `TickContext.displacement_mode` — the one non-graph input territory.py reads (§5). |
| `src/babylon/engine/simulation_engine.py` | 611 | `_SYSTEM_CLASSES` tuple (territory.py:328-364) — confirms tick position. |
| `src/babylon/engine/systems/territory_diagnostics.py` | 151 | Hex/county rollup helper. **Not called by `TerritorySystem.step()`** (grep-confirmed; its only importer is `sentinels/vocabulary/registry.py`) — out of scope for this port. |

**Not exercised by territory.py at all:** no `src/babylon/formulas/*` module, no `src/babylon/domain/*` module. This is the cleanest file-map in the estate — everything lives in one 378-line class.

**Reference BSL packs read for format** (all fully read):
- `rust/crates/babylon-tick/content/rules/metabolism.bsl` (412 lines) — D-record conventions, `defconst`, the `Currency × Ratio` scale-op, the bare-scaled-Int workaround.
- `rust/crates/babylon-tick/content/rules/vitality.bsl` (91 lines) — single-rule-per-position pattern, the "no bare non-integer literal" constraint, the Real-zero promotion trick, the Real→Int demotion blocker.
- `rust/crates/babylon-bsl/src/scenario.rs` lines 1-120 and `attribute_value`/`Atom::Currency` handling (lines 653-690, 317-330) — confirms Currency-typed field storage is refused at scenario load, every landed pack routes around it by declaring fields `int`.

## 2. COMPUTATION CATALOG (execution order, territory.py:102-105)

### Phase 1 — Heat dynamics (`_process_heat_dynamics`, territory.py:107-137)
- **(a)** HIGH_PROFILE territories accumulate heat linearly; everything else decays heat geometrically.
- **(b)** `new_heat = current_heat + high_profile_heat_gain` (territory.py:132) **or** `new_heat = current_heat * (1.0 - heat_decay_rate)` (territory.py:135); write via `_write_clamped(..., lo=0.0, hi=1.0)` (territory.py:137, clamp body at system_base.py:189: `max(lo, min(hi, value))`).
- **(c) Reads:** `TERRITORY.profile` (str→`OperationalProfile`, territory.py:123-126), `TERRITORY.heat` (default 0.0, territory.py:128).
- **(d) Writes:** `TERRITORY.heat`, clamped `[0,1]`.
- **(e) Defines:** `territory.heat_decay_rate` (0.1, `[0,1]`), `territory.high_profile_heat_gain` (0.15, `[0,1]`) — defines.yaml:235-236, `TerritoryDefines` territory.py(defines):17-28.
- **(f) Events:** none.

### Phase 2 — Eviction pipeline (`_process_eviction_pipeline`, territory.py:196-267)
- **(a)** Once heat crosses threshold, latch `under_eviction`; every tick thereafter the territory's rent spikes and a fraction of its population is displaced to a priority-ordered adjacent sink (or vanishes if none exists).
- **(b)** Latch: `if current_heat >= eviction_threshold and not under_eviction: graph.update_node(node.id, under_eviction=True)` (territory.py:236-238, one-way — `under_eviction` never reverts, confirmed by the law test `test_eviction_flag_never_reverts_once_set`). Rent spike: `rent_level *= rent_spike_multiplier` (territory.py:251, via `current_rent * rent_spike_multiplier`). Displacement: `displaced_pop = int(current_pop * displacement_rate)`; `new_pop = current_pop - displaced_pop` (territory.py:246-247). Sink transfer: `_find_sink_node` (territory.py:139-194) picks the highest-priority adjacent `ADJACENCY`-connected sink by `_PRIORITY_BY_MODE[mode]` (territory.py:66-82, 166-193); transfers are accumulated in a dict first, then applied (territory.py:227, 259-267) — the frozen system's own order-independence discipline.
- **(c) Reads:** `TERRITORY.heat`, `TERRITORY.under_eviction`, `TERRITORY.rent_level` (default 1.0), `TERRITORY.population` (default 0); `EdgeType.ADJACENCY` edges + target `TERRITORY.territory_type`; `TickContext.displacement_mode` (context.py:50, default `None`→`EXTRACTION`).
- **(d) Writes:** `TERRITORY.under_eviction`, `TERRITORY.rent_level`, `TERRITORY.population` (source and every sink territory).
- **(e) Defines:** `territory.eviction_heat_threshold` (0.8, `[0,1]`), `territory.rent_spike_multiplier` (1.5, `>0`, **unbounded above**), `territory.displacement_rate` (0.1, `[0,1]`) — defines.yaml:237,238,239.
- **(f) Events:** none.

### Phase 3 — Heat spillover (`_process_spillover`, territory.py:269-316)
- **(a)** Heat spills symmetrically across every `ADJACENCY` edge — each endpoint receives a fraction of the other's *pre-tick* heat (territory.py:279-284 explains this is deliberately symmetric per-unordered-pair, not directed).
- **(b)** `spillover_amounts[target] += source_heat * spillover_rate`; `spillover_amounts[source] += target_heat * spillover_rate` (territory.py:304-309); applied as `new_heat = min(1.0, current_heat + spillover)` (territory.py:315) — **note: this clamp is hand-written (upper-only), not `_write_clamped`**, unlike Phase 1's identical-domain `heat` write.
- **(c) Reads:** `TERRITORY.heat` on both endpoints of every `ADJACENCY` edge (node-type-filtered, territory.py:296-299).
- **(d) Writes:** `TERRITORY.heat` (all territories touched by an `ADJACENCY` edge).
- **(e) Defines:** `territory.heat_spillover_rate` (0.05, `[0,1]`) — defines.yaml:240.
- **(f) Events:** none.

### Phase 4 — Necropolitics (`_process_necropolitics` + `_suppress_organization`, territory.py:318-378)
- **(a)** `CONCENTRATION_CAMP` territories lose population every tick (elimination); `PENAL_COLONY` territories zero the organization of every connected `SocialClass`.
- **(b)** `new_pop = int(current_pop * (1.0 - decay_rate))` (territory.py:346). `_suppress_organization`: for every `EdgeType.TENANCY` edge whose target is this territory, `graph.update_node(edge.source_id, organization=0.0)` (territory.py:378).
- **(c) Reads:** `TERRITORY.territory_type`, `TERRITORY.population`; incoming `EdgeType.TENANCY` edges' `SOCIAL_CLASS` sources.
- **(d) Writes:** `TERRITORY.population` (camp only); `SOCIAL_CLASS.organization` (unconditional hard-set to `0.0`, penal colony only — cross-node-type write).
- **(e) Defines:** `territory.concentration_camp_decay_rate` (0.2, `[0,1]`) — defines.yaml:242.
- **(f) Events:** none.

**Events emitted by the whole system: zero.** Confirmed by grep — no `EventType`/`emit`/`.publish(` reference anywhere in territory.py. Unlike Vitality/Metabolism, TerritorySystem is entirely silent on the event bus.

## 3. TYPE INVENTORY

Runtime storage note first (load-bearing for everything below): `BabylonGraph.update_node` (`src/babylon/topology/graph.py:660-670`) is a **plain dict merge with no type coercion or quantization**. The `Currency`/`Intensity`/`Probability` Pydantic types (`models/types.py`) apply `SnapToGrid` (1e-5 grid, `kernel/math.py:41-61`) only when a `Territory` model is *instantiated* (scenario seed / `WorldState` round-trip) — never mid-tick. So all in-tick arithmetic below is raw Python `float`/`int` with no grid quantization.

| Attribute | Node type | Python model type | Domain | Category |
|---|---|---|---|---|
| `profile` | TERRITORY | `OperationalProfile` (StrEnum) | `{LOW_PROFILE, HIGH_PROFILE}` | **Enum discriminant** |
| `heat` | TERRITORY | `Intensity` | `[0.0, 1.0]` | unit-interval |
| `rent_level` | TERRITORY | `Currency` (= `Annotated[float, ge=0.0]`) | `[0.0, ∞)` | **unbounded real, money-semantic** |
| `population` | TERRITORY | `int` | `≥ 0` | integer |
| `under_eviction` | TERRITORY | `bool` | `{T,F}` | boolean latch |
| `territory_type` | TERRITORY | `TerritoryType` (StrEnum, 5 members) | closed set | **Enum discriminant** |
| `organization` | SOCIAL_CLASS | (elsewhere, `Coefficient`-shaped) | `[0,1]` typically | unit-interval (write-only here, hard-set to `0.0`) |
| `heat_decay_rate`, `high_profile_heat_gain`, `eviction_heat_threshold`, `displacement_rate`, `heat_spillover_rate`, `concentration_camp_decay_rate` (defines) | — | `float` | `[0.0, 1.0]` | unit-interval coefficients |
| `rent_spike_multiplier` (define) | — | `float` | `> 0.0`, **no upper bound** | **unbounded real coefficient** |

**Currency flag — not a hard STOP, but needs explicit handling.** `rent_level` is `Currency`-typed in the Python reference (`territory.py:136`), but the Python "Currency" is just an unbounded-above float, not BSL's `i128` micro-unit type — and crucially, `GraphSubstrate` attribute storage in BSL is *always* plain `f64`, and every landed pack (`metabolism.bsl`, `dispossession.bsl`) already routes around BSL's true `Currency` type by declaring money-like territory fields as `deffield ... int extensive` (confirmed: `content/scenarios/*.bscn` — `territory/wealth`, `territory/biocapacity` are all `int`, never `currency`; `scenario.rs:681-682` explicitly *refuses* a `Currency`-typed field's attribute value). **`rent_level` should follow this same precedent** — declare `territory/rent-level :type int`, read as `Value::Real` via `:field`. This is portable, not blocked — see §6.

**Enum discriminant flag — a genuine, previously-unnamed gap.** `deffield`'s closed six-type vocabulary is exactly `{int, bool, currency, probability, intensity, coefficient}` (`bsl-language.rst` §3.1) — **there is no `enum` row**. `Enum<T>` is explicitly listed as "a typechecker type that no `<type-name>` position can name" (§3.1) — it exists only for `NodeType`/`EdgeType` operands in queries, never as a per-node stored attribute. `profile` (2-valued) and `territory_type` (5-valued) have **no direct BSL field-storage representation today**. Workaround (content-modeling, not a language change, same class as Metabolism's D-2): `profile` fits trivially in one `bool` (`territory/is-high-profile`); `territory_type` needs either 5 mutually-exclusive `bool` fields or a single `int`-encoded ordinal compared via `=`/`!=`/`<`. Neither requires new grammar, but it is a genuine transcription decision with no existing precedent in a landed pack, and no Q-item in `reports/bsl-gap-analysis-2026-08-10.md` names it by name for Territory specifically — flag it as a fresh finding for the D-record.

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `float`), no libm transcendentals anywhere in territory.py — grep-confirmed zero `exp`/`log`/`sigmoid`/`pow` calls. Shapes, in execution order:

1. **Additive accumulation:** `current_heat + high_profile_heat_gain` (territory.py:132) — one add.
2. **Multiplicative decay:** `current_heat * (1.0 - heat_decay_rate)` (territory.py:135) — one subtract, one multiply. The bare `1.0` literal is a **BSL parser problem**: the "no bare non-integer literal" rule means `1.0` cannot be written directly; needs a `c`-suffixed `1c` or the Real-zero-promotion idiom already used twice in the read reference packs.
3. **Threshold comparison:** `current_heat >= eviction_threshold` (territory.py:236) — plain `>=`.
4. **Multiplicative rent spike:** `current_rent * rent_spike_multiplier` (territory.py:251) — the **flagged D-1-class hazard**: `rent_spike_multiplier`'s declared domain `(0, ∞)` is out-of-`[0,1]`, exactly the shape that blocked Metabolism's `entropy_factor` (§6).
5. **Real→Int demotion (×2):** `int(current_pop * displacement_rate)` (territory.py:246) and `int(current_pop * (1.0 - decay_rate))` (territory.py:346) — truncating cast. *(Adjudication: `floor` landed in #489; both sites are non-negative so trunc ≡ floor — expressible today.)*
6. **Symmetric accumulation with a running dict:** `spillover_amounts.get(k, 0.0) + source_heat * spillover_rate` (territory.py:304-309) — order-independent by construction (collect-then-apply), which is exactly BSL's own per-position same-pre-state semantics — a genuinely *favorable* structural match, not a hazard.
7. **Clamps:** `max(lo, min(hi, value))` (system_base.py:189, Phase 1 only) vs. the hand-written `min(1.0, current_heat + spillover)` (territory.py:315, Phase 3, upper-only) — **two different clamp implementations for the conceptually-identical `[0,1]` `heat` field**; the pack must transcribe both shapes faithfully (port-as-is), expressed as nested `if` per landed-pack precedent (no scalar `min`/`max` in the grammar).
8. **Currency-mixing multiply, unresolved class:** none of Territory's multiplies are `Currency × X` in the BSL sense (since `rent_level` is `:field`-sourced → always `Value::Real`), so the *actual* Currency operator-table restrictions never bind here — the risk is entirely the `Ratio`-domain-const problem in item 4.

No exp/log/sigmoid anywhere — **this system has zero libm-nondeterminism hazard**, unlike Metabolism.

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 2.0** (territory.py:59), confirmed against `_SYSTEM_CLASSES` (`simulation_engine.py:328-364`): `VitalitySystem (1.0) → TerritorySystem (2.0) → SubstrateSystem (2.5) → ProductionSystem (3.0) → ...`. Only Vitality runs before it this tick.
- **Reads from a same-tick prior system: none.** Vitality only mutates `SOCIAL_CLASS.wealth`/`.active`/`.population` — grep-confirmed no overlap with anything Territory reads. Territory's every read is its own prior-tick persisted state or scenario-seed state.
- **Writes consumed later this tick / downstream ticks:**
  - `TERRITORY.population` — read by **13 other systems** (grep-confirmed): `faction_influence.py`, `vol2_circulation.py`, `market_scissors.py`, `electoral.py`, `substrate.py` (@2.5, immediately next), `epistemic_horizon.py`, `production.py` (@3.0), `policy.py`, `reserve_army.py` (@5.0), `lifecycle.py` (@7.0), `metabolism.py` (@13.0), `sovereignty.py`, `dispossession_events.py` (@10.0), `struggle.py`. This is Territory's single most load-bearing output.
  - `SOCIAL_CLASS.organization` (from `_suppress_organization`, PENAL_COLONY only) — read downstream by `control_ratio.py` (@12.0), `struggle.py` (@16.0), `survival.py` (@15.0), `allegiance.py` (@17.42). A genuine cross-node-type, cross-system channel.
  - `TERRITORY.heat`, `TERRITORY.rent_level`, `TERRITORY.under_eviction` — grep-confirmed **read by no other System**. These are terminal/observational outputs, not engine-internal channels.
- **Context/service usage with no BSL equivalent:** `context.get("displacement_mode", DisplacementPriorityMode.EXTRACTION)` (territory.py:224) reads `TickContext.displacement_mode` (`engine/context.py:50`, `Optional[DisplacementPriorityMode]`, default `None`). This is a per-run **test/API-only override** — no production code path in `simulation_engine.py` ever sets it (grep-confirmed), and the `TerritoryDefines` AUTO-mode thresholds meant to drive it dynamically (`elimination_rent_threshold`, `elimination_tension_threshold`, `containment_rent_threshold`, `containment_tension_threshold` — defines.yaml:244-247) are **never read anywhere** ("reserved for future use" per the docstring at `config/defines/territory.py:72`). Effectively the mode is a hardcoded `EXTRACTION` in every live simulation. **This is a WS1-ledger item**: since the value is provably always `EXTRACTION` on every real code path, the honest port declares it a `:const` (Metabolism D-2-style "provably uniform" reasoning), dropping the override entirely and recording why.
- **Dormancy nuance not visible in the R8 table's one-line verdict.** The canonical `SCENARIOS` factories (`_legacy.py:728-732`, `:986-989`) *explicitly* state: **"This scenario emits NO ADJACENCY edges"** ("no real county-adjacency reference source exists," Constitution III.11 honesty). So while the R8 gap report marks Territory "Dormant on canonical: No" (live), that is only true for **heat accumulation/decay** (Phase 1, self-scoped, always live) and **PENAL_COLONY suppression** (Phase 4, needs only `TENANCY`, which *is* seeded — but no canonical scenario seeds a `PENAL_COLONY` territory either, see `tools/regression_scenarios.py:2678`). **Heat spillover (Phase 3) and ADJACENCY-based sink routing (Phase 2's `_find_sink_node`) are structurally dormant on every canonical `qa:regression` scenario** — no conformance vector in the current estate exercises them end-to-end. A port's conformance fixtures will need to be hand-built (as Metabolism's `.bscn` fixtures are), not harvested from the canonical scenarios.

## 6. BLOCKER ASSESSMENT (agent's table — superseded rows marked)

| Computation | Verdict | Detail |
|---|---|---|
| Phase 1 heat accumulate/decay (territory.py:107-137) | **PORTABLE NOW** | Both `heat_decay_rate`/`high_profile_heat_gain` are `[0,1]`-domain — trivial `c`-suffixed `defconst`, exact precedent in every landed pack. `profile` needs the enum→`bool` workaround (§3) — a D-record, not a blocker. Clamp: nested-`if`, matching the landed-pack clamp convention. |
| Phase 2 eviction latch + rent spike (territory.py:236,251) | **PORTABLE WITH D-RECORD** | The latch itself is a plain `if`+`update-node`, trivial. `rent_level * rent_spike_multiplier` hits the **same D-1-class hazard** Metabolism's `entropy_factor` hit: the #500 scale-op only helps a `Value::Currency` operand, and `rent_level` is `:field`-sourced → always `Value::Real` — so **the scale-op landing does NOT close this gap for Territory's actual arithmetic**. Deviation: the identical bare-scaled-Int workaround, same accepted-deviation class under ADR183 §5.4. *(But see the adjudication: latch+spike must not land without displacement — the pipeline is one behavior.)* |
| Phase 2 population displacement (territory.py:246) | ~~BLOCKED~~ **SUPERSEDED: expressible** | The agent cited vitality.bsl's pre-#489 header; `floor` landed in #489 (`declarations.rs:110`). Non-negative operand → trunc ≡ floor. |
| Phase 2 sink selection (`_find_sink_node`, territory.py:139-194) | **BLOCKED — query evaluation** | Needs typed `neighbors`/incidence accessors + `select-max`-class priority selection, all of which parse and typecheck but **refuse at evaluation** (`evaluator.rs:421-430`, "Task 16 / the Phase-2 query evaluator"). |
| Phase 2 population transfer to sink (territory.py:259-267) | **BLOCKED — query evaluation** | Cross-node write targeting: `update-node` accepts a computed `NodeRef` (structural_verbs.rs:637), but *producing* that ref requires the refusing query verbs; `for-each` in effect position also refuses (structural_verbs.rs:258). |
| Phase 3 heat spillover (territory.py:269-316) | **BLOCKED — query evaluation, favorably reformulable** | The frozen collect-then-apply pattern maps cleanly onto a pull-side `fold` over `neighbors` — mathematically identical, not a deviation — but `fold`/`neighbors` refuse at evaluation today. Also dormant on every canonical scenario (§5), so no conformance oracle exists yet either way. |
| Phase 4 CONCENTRATION_CAMP decay (territory.py:344-347) | ~~BLOCKED~~ **SUPERSEDED: expressible** | Same floor resolution as Phase-2 displacement. |
| Phase 4 PENAL_COLONY suppression (territory.py:349-378) | **BLOCKED — query evaluation** | Incoming-`TENANCY` incidence query + cross-node-type write — same refusing surface. |
| `territory_type`/`profile` enum reads throughout | **PORTABLE WITH D-RECORD** | No `deffield` enum row exists (§3). Workaround: `bool`-per-value (profile) or `int`-ordinal (territory_type), a content-modeling decision requiring its own D-record documenting the chosen mapping. |
| `TickContext.displacement_mode` (territory.py:224) | **PORTABLE WITH D-RECORD** | Provably always `EXTRACTION` on every production code path (§5) — declare as a Metabolism-D-2-style "provably uniform" `:const`. |

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_territory_system.py` | 1303 | **Primary conformance oracle.** Exhaustive per-phase unit coverage: heat dynamics (gain/decay/clamp-at-1/clamp-at-0), eviction pipeline (threshold/rent-spike/displacement), spillover (symmetric-across-one-edge/no-spillover-from-non-adjacent/capped-at-1), sink selection (all three priority modes + fallbacks), population transfer (transfer-to-sink vs. delete-when-no-sink), necropolitics (camp elimination + penal suppression), displacement-mode overrides (including `test_context_displacement_mode_is_respected` — confirms `context.displacement_mode` IS exercised, just never by production code). Every scenario here is a candidate conformance vector for the BSL port's own `.bscn` fixtures. |
| `tests/unit/engine/laws/test_law_territory_system.py` | 236 | **Property-based invariant contracts**: `test_heat_stays_within_unit_interval`, `test_population_never_goes_negative`, `test_eviction_flag_never_reverts_once_set`, `test_social_class_without_tenancy_edge_is_untouched`. Exactly the behavioral-contract laws the BSL port's conformance scenarios should re-prove independent of bit-exactness. |
| `tests/contract/state_ai/test_territory_contract.py` | 691 | AI/state-observer contract surface — documents what fields the AI-narrative layer expects to read off `Territory` post-tick. |
| `tests/integration/test_territory_edge_serialization.py` | 107 | `ADJACENCY`/`TENANCY` edge round-trip through graph serialization. |
| `tests/unit/engine/test_territory_diagnostics.py` | 132 | Tests the out-of-scope `territory_diagnostics.py` rollup helper — not TerritorySystem itself. |
| `tests/unit/models/test_territory.py` | 574 | `Territory` Pydantic model validation (field bounds, `SnapToGrid` quantization) — schema-level, not tick-behavior. |
| `tests/unit/projection/test_territory_anchor.py` | 131 | Projection/anchor layer — `observe()`-page rendering, not engine math. |
| `tests/unit/state_ai/test_territory_effects.py` | 936 | AI-effects layer consuming Territory state — narrative, not conformance. |

**qa:regression byte-gate coverage.** `tools/regression_test.py::graph_content_hash` (lines 924-964) hashes **every** node/edge attribute of the `WorldState→graph` projection — so any change to `TerritorySystem`'s outputs on any canonical scenario **is caught by the byte-identical hash gate**, even though no scenario names Territory explicitly. Given §5's dormancy finding, that coverage is real only for Phase 1 (heat) — no canonical territory is seeded `HIGH_PROFILE`/`PENAL_COLONY`/`CONCENTRATION_CAMP` (declared gap, `tools/regression_scenarios.py:2678-2683`) — **Phases 2-4's special-type and ADJACENCY-dependent paths have zero canonical-scenario coverage** and will need dedicated `.bscn` conformance fixtures analogous to `metabolism-conformance.bscn`/`dispossession-conformance.bscn`, built by hand rather than harvested from the canonical estate.
