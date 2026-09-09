# ProductionSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `ProductionSystem` (`src/babylon/engine/systems/production.py`, 268 lines,
tick position 3.0) is a single loop over every `SOCIAL_CLASS` node that iterates producer roles
(`PERIPHERY_PROLETARIAT`, `LABOR_ARISTOCRACY`), reads each worker's tenanted territory via a
`TENANCY` edge, computes `produced_value = labor_power × population × bio_ratio`, and routes it
either to the worker's own wealth (direct producers) or to a `WAGES`-linked employer (Amin/Wallerstein
labor-aristocracy routing) — followed by a second loop broadcasting per-territory
`extraction_intensity` for `MetabolismSystem`'s biocapacity depletion. A verified, load-bearing
defect was found: the tensor-registry-driven `effective_labor_power` branch reads
`territory_attrs.get("fips_code")`, but `Territory`'s real field is `county_fips` — this branch is
**provably dead on every scenario in the estate**, including the one scenario purpose-built to
exercise it. The system emits zero events, has zero libm hazards, and is genuinely live (not
dormant) on the flagship `imperial_circuit` canonical scenario. One real blocking gap was found
(graph-level scratch state, `la_production`), one genuine reformulation challenge (push- vs
pull-style wealth accumulation across a shared employer), and the rest of the arithmetic is
directly portable using precedent already landed for other systems.

**Verdict: PORTABLE WITH D-RECORDS for most computations; BLOCKED on one channel (graph-level
scratch state carrying `la_production` to `ImperialRentSystem` — no `GraphSubstrate` construct for
it, only a documented-but-unimplemented "carrier NodeType" design ruling) and UNVERIFIED-IN-PRACTICE
on one reformulation (pull-style grouped fold replacing the frozen system's sequential push-style
employer-wealth accumulation). No RESERVED-LINE surface requiring Director escalation, though the
producer-role routing structure is the direct engine mechanism of the Amin/Wallerstein imperial-bribe
model central to the National Question line — described here, not touched.**

**UPDATE (2026-08-12) — the port itself has landed.** *(Sizes below are AS ORIGINALLY LANDED —
the fix round two paragraphs down grew the pack to FIVE rules and the fixture to nine social
classes/five territories/twelve edges; kept here unedited as the historical record of what this
task originally shipped.)* The Production port plan
(`docs/superpowers/plans/2026-08-12-production-port-plan.md`, Tasks 1-5, branch
`feat/production-port-bsl`) ships the actual transcription this inventory's own Adjudication
(below) narrowed but did not build: `production-conformance.bscn` (eight social classes, four
territories, eleven edges — every conformance case this report's own §5/§6/§7 named, in one
hand-built world) plus its frozen-engine mirror (`production_conformance.py`, the STRUCTURE
oracle, ADR183), and `production.bsl`'s four rules — `p1-direct-production`,
`p2-employed-routing`, `p3-employed-fallback`, `p4-extraction-intensity` — byte-ordered at ONE
anchor position, deliberately relying on D116's cross-rule divergence (register row D132). The
scout dossier (`reports/production-bsl-surface-facts-2026-08-12.md`, read before this train
started) corrected this inventory's own Adjudication on the two points that mattered most: `dev`'s
own `accumulation_into_a_shared_target_reduces_in_subject_order_and_keeps_every_contribution`
proof (`tick.rs:994-1076`) directly discharges the employer-wealth accumulation this inventory
rated **UNVERIFIED-IN-PRACTICE** — no int-ordinal `role` workaround was ever needed, since `role`
gating stays on the rule's own SUBJECT side (`(binding role :field social-class/role)`, D101,
legal before D102 even landed) rather than inside a fold body reading a neighbour's role. **This
port DISCHARGES the `la_production` channel Adjudication correction 5 reinforced as "BLOCKED,
worse than described" — not via the carrier-`NodeType`/`the` route this inventory and its
Adjudication both examined (still Slice-2 unserved), but by recognizing the channel was never a
genuine graph-scope total at all** — per-node data (keyed by worker node id) wearing a graph-scope
costume — and an ordinary `deffield` (`social-class/production-value`) dissolves the blocker
without touching graph scope, `the`, or any unserved machinery.

**The extraction-intensity broadcast — this Adjudication's own correction 2 narrowed it from
PORTABLE NOW to "portable only under an int-ordinal `role` encoding"; the scout dossier found a
DIFFERENT blocker neither this Adjudication nor its own correction caught, and the port resolves
it a third way.** Correction 2 is right that D102 (then unconditional) blocked a neighbour-role
`field-of` read inside a fold body — but D102 is now discharged (verified independently by the
scout dossier §2) AND, more importantly, the port's own design never needed a neighbour-side role
read at all: the filter moves onto the per-node `social-class/production-value` field p1-p3
already compute and filter via their OWN `when` guards, so p4's fold body is the bare accessor
`field_ref_for` requires (§3.4's compound-expression restriction — a DIFFERENT, independent law
from D102, which the scout dossier's §5 is the first place to name precisely). Register row D138
records this as the scout dossier's own headline correction, transcribed.

**AS ORIGINALLY LANDED (kept in present tense below as the historical record of the FIRST design —
CORRECTED two paragraphs down, MINOR-D fix round: this is no longer what the pack does; read
on).** A genuinely NEW divergence, discovered and measured during Task 4's own test-writing, that
neither this inventory, its Adjudication, nor the scout dossier predicted: `worker-pp-two-lands`
(a fixture node holding TWO `TENANCY` edges, built specifically to exercise the D45 select-max
tiebreak this inventory's own correction 8 named as expressible) turns out to exercise a SECOND,
independent divergence at `p4`: the per-territory fold reads `production-value` off EVERY
`TENANCY`-incident neighbour of a territory, regardless of which single territory that worker's
OWN bio-ratio computation selected — so `p4` credits its one computed value to BOTH territories'
extraction-intensity totals, where the frozen engine's `territory_production[territory_id] +=
produced_value` credits exactly one. Measured, not assumed: `t-beta`'s own extraction-intensity
(`0.01730769230769231`) genuinely diverges from the frozen mirror's own printed value
(`0.009615384615384616`), while `t-alpha`'s agrees bit for bit — filed as register row D136,
distinct from D135 (the bio-ratio tiebreak comparison this inventory's own correction 8 and the
scout dossier's §4 both already named, which turns out NON-discriminating on this specific
fixture).

**AS ORIGINALLY LANDED (same caveat — CORRECTED below).** The landed design supersedes correction 3
outright (the D116 multi-rule-pack analysis, "safe on the substance"), not merely updates it — the
port's actual design deliberately depends on the cross-rule visibility correction 3 argued the
design didn't need. The landed `p4-extraction-intensity` reads `p1`/`p2`/`p3`'s own same-tick
writes to `production-value`, exactly the shape the scout dossier's own §9 reanalysis
(independently reaching the same conclusion this update states) predicted the extraction-fold
reformulation would need once it moved off a pre-tick-only design.

**CORRECTED (fix round, 2026-08-12, adversarial verification) — the TWO paragraphs immediately
above, as originally written, described the LANDED design accurately at the time; they no longer
do.** Register row D136 corrects their own claim that `p4` reads `production-value` via a
per-territory fold, and that this fold's own double-count had "no fixture-level or `.bsl`-level
fix available within a port-as-is mandate," in place — this report does not repeat that correction
verbatim a second time. That claim was fiction, and adversarial verification caught it. A scratch probe
built against the already-landed grammar — a producer-side PUSH design, no new GRAMMAR construct
at all (the probe DOES mint a second field, `territory/production-total` — licensed on the same
precedent that already licensed `social-class/production-value`, this update's own second
paragraph above; denying that license was exactly the fabrication) — loaded, ran, and reproduced
the frozen engine's own `t-beta` value (`0.009615384615384616`) bit for bit on first execution.
The landed correction, CURRENT as of this fix round: `territory/production-total` (`int
extensive`, seeded `0` on every territory) replaces the pull fold; a new rule, `production/
p0-production-total-reset`, zeroes it every tick (byte-sorted before p1); `p1`/`p2`/`p3` each gain
a THIRD effect pushing their own output onto the SAME D45-tiebreak-selected territory ref their
`bio`/`max-bio` bindings already compute; `p4` reads the result back via a plain `:field` binding
— no fold, and `p4` never reads `social-class/production-value` at all any more (the two paragraphs
above's own "reads production-value" claim is the part of "originally landed" that changed).
`t-beta`'s measured extraction-intensity now agrees with the frozen mirror bit for bit — this
correction DISCHARGES the divergence this update once reported as permanent, rather than merely
narrowing it. Full record (including the one honestly-recorded semantic cost — a tenancy-less
producer now does not fire at all, exercised by no fixture node today):
`ai/decisions/ADR200_production_port_handoff.yaml` item 13.

**What this update does NOT resolve — recorded, not fixed, the same posture Territory's own port
handoff uses.** The `1.0c` coefficient-boundary fragility for `economy/base-labor-power-annual`
(D137) retires only with #502 workstream 3's `Real x Ratio` operator or unbounded-domain
coefficient storage. The pack omits the `fips_code`/`county_fips` dead tensor branch (D133) rather
than repairing it, per the Director's port-as-is ruling. The D135 multi-tenancy bio-ratio tiebreak
comparison stays deliberately unresolved (both engines' answers stand, independently correct; this
fixture's own tiebreak happens not to discriminate between them). The correction two paragraphs
above CLOSES D136 (the extraction-intensity double-count) — it no longer belongs in this
"not resolved" list, which is precisely what this fix round corrected.

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/production.py` | 268 | **The target.** `ProductionSystem`, one `step()` method plus three helpers (`_find_tenancy_target`, `_find_employer`, `_update_extraction_intensities`). Read completely, line by line. |
| `src/babylon/config/defines/economy_basic.py` | `EconomyDefines` class 149-~230; `base_labor_power` field 168-173 | Coefficient source for `services.defines.economy.base_labor_power` (production.py:112). |
| `src/babylon/config/defines/tunables.py` | `TimescaleDefines` 53-102; `weeks_per_year` field 77-81 | Coefficient source for `services.defines.timescale.weeks_per_year` (production.py:113). |
| `src/babylon/data/defines.yaml` | `economy:` block 70-~110 (`base_labor_power` line 73); `timescale:` block 372-375 (`weeks_per_year` line 374) | Player-editable YAML values. |
| `src/babylon/domain/economics/tensor_registry.py` | 618 | `TensorRegistry.get(fips, year)` (lines 179-221) — the cache lookup production.py:168 calls. Provably unreachable in this system (§2, §5). |
| `src/babylon/domain/economics/tensor.py` | 558 | `NoDataSentinel` (45-130, falsy marker) and `ValueTensor4x3.total_v` (`@computed_field`, 363-374: sum of `.v` across four Marxian departments) — the value production.py:172 would read if the branch were reachable. |
| `src/babylon/models/entities/social_class.py` | 522; `role` 296-299, `wealth` 308-311, `active` 380-383, `population` 406-410 | `SocialClass` entity — field types/domains for every `SOCIAL_CLASS` attribute this system reads or writes. |
| `src/babylon/models/entities/territory.py` | 352; `biocapacity` 155-159, `max_biocapacity` 160-164, `extraction_intensity` 171-176, `county_fips` 81-91 | `Territory` entity — confirms the real FIPS field is `county_fips`, **not** `fips_code` (§2 defect). |
| `src/babylon/models/enums/social.py` | 211; `SocialRole` 12-64 (8 members) | The class-position enum this system filters/routes on. |
| `src/babylon/models/enums/topology.py` | 253; `NodeType.SOCIAL_CLASS`/`TERRITORY` 61-62, `EdgeType.TENANCY`/`WAGES` 104-106 | Node/edge type discriminants used in every `query_nodes`/`query_edges` call. |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase` abstract base. **Not exercised beyond the abstract `step()` contract and the class-level `creates_value`/`invariants`/`phase` declarations** — this system calls `graph.update_node` directly, never `self._write_clamped` or `self._publish` (§4). |
| `src/babylon/kernel/graph_protocol.py` | 494; `query_nodes` 258-276, `query_edges` 278-298, `get_node` 77-86, `update_node` 88-98, `get_graph_attr`/`set_graph_attr` 350-372 | `GraphProtocol` signatures this system calls. |
| `src/babylon/topology/adapters/query_mixin.py` | 146; `query_nodes` 34-68, `query_edges` 70-~110 | Concrete iteration: `for node_id in self._graph.nodes` — deterministic rustworkx node-index order. |
| `src/babylon/topology/graph.py` | 1033; `update_node` 660-670, `get_graph_attr`/`set_graph_attr` 892-898 | Concrete `BabylonGraph`. `update_node` is a **plain dict merge, no type coercion or quantization mid-tick** (same finding as Territory/Solidarity). `get_graph_attr`/`set_graph_attr` back a plain `dict[str, Any]` (`self._graph_attrs`), entirely separate from node/edge storage. |
| `src/babylon/kernel/tick_partition.py` | `TickPartition.MATERIAL_BASE`, lines 18-28 | production.py:67 declares this partition. |
| `src/babylon/kernel/system_protocol.py` | `ContextType`, line 16 | `type ContextType = "TickContext"` — `context.tick` read at production.py:123. |
| `src/babylon/engine/invariants.py` | `NonNegativeWealth`, 103-130 | `self.invariants = [NonNegativeWealth()]` (production.py:86) — a Spec-040 declaration, self-tested only (§5). |
| `src/babylon/engine/phase.py` | `Phase` IntEnum, 21-38 | `self.phase = Phase.PRODUCTION` (production.py:87) — same self-tested-only status. |
| `src/babylon/engine/simulation_engine.py` | `_SYSTEM_CLASSES`, 328-363 | Confirms tick position 3.0 and full 34-system order (§5). |
| `src/babylon/domain/economics/tick/graph_bridge.py` | `resolve_county_identity`, 44-~65 | Docstring states, as the documented ground truth: *"The county identity of a territory lives in its `county_fips` attribute and nowhere else"* — the direct evidence for the `fips_code` defect (§2). |
| `src/babylon/kernel/sim_clock.py` | 56; `SIM_EPOCH_YEAR=2010` (19), `WEEKS_PER_YEAR=52` (22, module-local constant), `_` formula (55) | A **third**, independently-hardcoded copy of the "epoch year from tick" formula this system's own comment (production.py:116-122) claims to reconcile against (§2, §4). |
| `src/babylon/domain/economics/tick/system/__init__.py` | `_determine_year`, 409-423 | `TickDynamicsSystem`'s own copy of the formula, default `base_year=2010` — diverges from production.py's own default of `2022` (§2). |

**Not exercised by production.py at all:** no `src/babylon/formulas/*` module call (unlike Vitality/
Metabolism). The only `domain/economics/*` dependency is the tensor-registry branch, and that branch
is provably dead (§2, §5) — so in practice this system's live computation touches no `domain/`
module either, the same "cleanest file map" shape Territory's inventory found for itself.

**Reference BSL/spec text read for this inventory** (all read in full for the cited ranges):
`rust/crates/babylon-bsl/src/declarations.rs` (deffield type vocabulary, `DECLARABLE_INTRINSICS`,
`defenum`/enum-`deffield` parsing); `rust/crates/babylon-bsl/src/scenario.rs` (Currency-storage
refusal); `rust/crates/babylon-bsl/src/evaluator.rs` (`eval_fold`/`eval_selection`, `for-each`
pre-state semantics); `rust/crates/babylon-tick/src/lib.rs` (`TickReport` field set — no event log);
`docs/reference/bsl-language.rst` §2.4 (conditions grammar), §2.7 (expressions/folds/guards
grammar, nested-fold `:as` worked example), §3.6 (graph-scope-state draft ruling, R9 chapter C3);
`rust/crates/babylon-tick/content/scenarios/organization-foundation.bscn` and
`content/rules/organization.bsl` (enum-`deffield` precedent, `defenum`/`:field`/`=` pattern);
`rust/crates/babylon-tick/content/scenarios/vitality-conformance.bscn` (the `SOCIAL_CLASS`-typed
bool→int precedent this system needs directly for `active`).

## 2. COMPUTATION CATALOG (execution order, `production.py:step`, lines 91-268)

### Computation 1 — Labor-power annualization + tensor-year setup (`production.py:111-129`)
- **(a)** Converts the annual `base_labor_power` coefficient to a per-tick (weekly) rate, resolves
  the current tick number, and initializes two accumulator dicts used later in the loop.
- **(b)** `base_labor_power = annual_labor_power / weeks_per_year` (production.py:114) — one
  division. `tick: int = context.tick` (production.py:123).
- **(c) Reads:** `services.defines.economy.base_labor_power`, `services.defines.timescale.weeks_per_year`,
  `context.tick`.
- **(d) Writes:** none (local variables only).
- **(e) Defines:** `economy.base_labor_power` (default `1.0`, `[0.0, ∞)`, defines.yaml:73,
  economy_basic.py:169-173); `timescale.weeks_per_year` (default `52`, `[1, ∞)` int, defines.yaml:374,
  tunables.py:77-81).
- **(f) Events:** none.

### Computation 2 — Per-worker production (`production.py:131-204`, the core loop)
- **(a)** For every active `SOCIAL_CLASS` node with a producer role, finds the worker's tenanted
  territory, computes a biocapacity ratio, scales by demographic block size, computes production
  value, and routes it to the worker's own wealth (direct producers) or to a `WAGES`-linked employer
  (employed producers — the Labor Aristocracy).
- **(b)** Gates (production.py:135-146): skip if `not attrs.get("active", True)`; skip if
  `role not in _PRODUCER_ROLES` (the 2-member subset `{PERIPHERY_PROLETARIAT, LABOR_ARISTOCRACY}` of
  the 8-member `SocialRole` enum, production.py:46-52); skip if `_find_tenancy_target` returns `None`
  (a linear scan of every `TENANCY` edge for `edge.source_id == worker_id`, production.py:212-225).
  Formula (production.py:151-175): `bio_ratio = 0.0 if max_biocapacity <= 0 else biocapacity /
  max_biocapacity` (zero-guarded division); `population = attrs.get("population", 1)`;
  `effective_labor_power` defaults to the weekly `base_labor_power` from Computation 1, then is
  **provably never overridden** by the tensor-registry branch (production.py:160-172) — see the
  defect note below; `produced_value = (effective_labor_power * population) * bio_ratio`
  (production.py:175, two multiplies). Routing (production.py:179-198): if
  `role in _DIRECT_PRODUCER_ROLES` (`PERIPHERY_PROLETARIAT`), `graph.update_node(node.id,
  wealth=current_wealth + produced_value)` — a self-write. If `role in _EMPLOYED_PRODUCER_ROLES`
  (`LABOR_ARISTOCRACY`), `_find_employer` (production.py:227-244, a linear scan of every `WAGES`
  edge for `edge.target_id == worker_id`, returning `edge.source_id`) locates the employer; if found,
  `graph.update_node(employer_id, wealth=employer_wealth + produced_value)` — a cross-node write —
  and `la_production[node.id] = produced_value` is recorded; if no employer is found, the worker
  falls back to receiving its own production directly (production.py:195-198, UNVERIFIED whether
  this fallback branch is ever live on a canonical scenario — see §5). Accumulation
  (production.py:200-204): `territory_production[territory_id] = territory_production.get(territory_id,
  0.0) + produced_value` whenever `produced_value > 0`.
- **VERIFIED DEFECT, load-bearing:** `fips_code = territory_attrs.get("fips_code")`
  (production.py:164) is **always `None`** on every real scenario. `Territory`'s Pydantic model
  (`models/entities/territory.py:81-91`) declares the field `county_fips`, never `fips_code`;
  `WorldState.to_graph()` stamps every territory node via `Territory.model_dump()`
  (`models/world_state.py:746`), so a live territory node's attribute dict never contains the key
  `fips_code`. `resolve_county_identity`'s own docstring states the ground truth directly: *"The
  county identity of a territory lives in its `county_fips` attribute and nowhere else"*
  (`domain/economics/tick/graph_bridge.py:47-48`). The ONLY place `fips_code` is ever written onto a
  territory graph node anywhere in the repository is the hand-stamped unit-test fixtures
  (`tests/unit/engine/systems/test_production.py:350,373,423,428,459,483`,
  `graph.nodes["T1"]["fips_code"] = "26163"`), which bypass the Pydantic model entirely — the exact
  CLAUDE.md "attribute-shape" gotcha (a fixture stamping an attribute a node's model doesn't declare
  gives a green test over dead production code). **This holds even for the one canonical scenario
  purpose-built to exercise this path**: `single_county` (`engine/scenarios/single_county.py`)
  stamps `county_fips="26163"` on its territory (line 116) and its module docstring states it is
  "the smallest graph where the Vol III financial layer fires through the production path"
  (single_county.py:1-2), and `tools/regression_test.py::build_single_county_overrides` (215-256)
  hydrates a REAL `tensor_registry` for it from a committed fixture — but because production.py
  reads `fips_code`, not `county_fips`, `tensor_registry.get(...)` is never called even here.
  `effective_labor_power` is 100% of the time the Computation-1 fallback value on every scenario in
  the estate. Port-as-is law: this is transcribed as a genuine defect, not silently repaired (§6).
- **(c) Reads:** `SOCIAL_CLASS.active` (default `True`), `.role`, `.population` (default `1`),
  `.wealth` (default `0.0`); `TERRITORY.biocapacity` (default `0.0`), `.max_biocapacity` (default
  `1.0`), `.fips_code` (verified always absent — dead read); `EdgeType.TENANCY` edges;
  `EdgeType.WAGES` edges; `services.tensor_registry` (`getattr` with `None` default — declared on
  `ServicesProtocol`, `kernel/services.py:55`, but its lookup path is unreachable per above);
  `graph.get_graph_attr("base_year", 2022)` (dead, same branch).
- **(d) Writes:** `SOCIAL_CLASS.wealth` (direct producer's own node, or the employer's node for
  employed producers).
- **(e) Defines:** none beyond Computation 1's `base_labor_power`/`weeks_per_year` (already resolved
  before the loop).
- **(f) Events:** none.

### Computation 3 — LA-production ledger publish (`production.py:207`)
- **(a)** Publishes the per-worker production-value map for employed (Labor-Aristocracy) producers
  as a **graph-level** attribute, for `ImperialRentSystem`'s wages phase to read back later this
  same tick.
- **(b)** `graph.set_graph_attr("la_production", la_production)` — a `dict[str, float]` keyed by
  worker node id, stored outside node/edge storage entirely.
- **(c) Reads:** the `la_production` dict accumulated in Computation 2.
- **(d) Writes:** graph-level attribute `la_production` (not a node or edge attribute — see §5's
  channel note and §6's blocker row).
- **(e) Defines:** none.
- **(f) Events:** none.

### Computation 4 — Extraction-intensity broadcast (`_update_extraction_intensities`, called
production.py:210, defined 246-268)
- **(a)** A second, independent loop over every `TERRITORY` node, broadcasting the total production
  extracted from it this tick as a normalized `[0,1]` intensity — the signal `MetabolismSystem`
  consumes to deplete biocapacity.
- **(b)** `total_production = territory_production.get(node.id, 0.0)`; `max_biocapacity =
  attrs.get("max_biocapacity", 100.0)`; `intensity = min(1.0, total_production / max_biocapacity) if
  max_biocapacity > 0 else 0.0` (production.py:267, zero-guarded division + upper-only clamp);
  `graph.update_node(node.id, extraction_intensity=intensity)`.
- **(c) Reads:** `TERRITORY.max_biocapacity` (default `100.0` — note, a **different default** than
  Computation 2's own read of the same field, `1.0`, production.py:152 vs :265 — both are Python
  `.get()` fallbacks only used when the attribute is genuinely absent from the node, so this is an
  inert but verbatim-transcribable inconsistency); the `territory_production` dict built in
  Computation 2.
- **(d) Writes:** `TERRITORY.extraction_intensity`, all territories (including those with zero
  production this tick, which get `intensity=0.0`).
- **(e) Defines:** none — the `1.0` clamp ceiling is a bare literal, not a `GameDefines` coefficient.
- **(f) Events:** none.

**Events emitted by the whole system: zero.** Grep-confirmed: no `EventType`/`event_bus`/`.publish(`/
`_publish(` reference anywhere in production.py. Matches Territory's finding, not Vitality's/
Solidarity's.

## 3. TYPE INVENTORY

Runtime storage note (load-bearing, identical finding to Territory/Solidarity):
`BabylonGraph.update_node` (`topology/graph.py:660-670`) is a plain dict merge with **no type
coercion or quantization mid-tick**. Pydantic's `SnapToGrid` (1e-6 grid, `kernel/math.py:16-17,41-61`
— `_PRECISION: int = 6`) applies only at model instantiation (scenario seed / `WorldState`
round-trip), never mid-tick. All in-tick arithmetic below is raw Python `float`/`int`.

| Attribute | Node type | Python model type | Domain | Category |
|---|---|---|---|---|
| `role` | SOCIAL_CLASS | `SocialRole` (StrEnum, 8 members) | closed set | **Enum discriminant** |
| `active` | SOCIAL_CLASS | `bool` | `{T,F}` | **boolean** |
| `population` | SOCIAL_CLASS | `int`, `ge=0` | `≥ 0` | extensive integer |
| `wealth` | SOCIAL_CLASS | `Currency` (`Annotated[float, ge=0.0]`, `SnapToGrid`) | `[0.0, ∞)` | **unbounded real, money-semantic**, extensive |
| `biocapacity` | TERRITORY | `Currency` | `[0.0, ∞)` (dynamically ≤ `max_biocapacity` via Metabolism's own clamp, `metabolism.py:116`, not by Pydantic) | unbounded real |
| `max_biocapacity` | TERRITORY | `Currency` | `[0.0, ∞)` | unbounded real |
| `county_fips` (the real field; `fips_code` read at production.py:164 does not exist) | TERRITORY | `str \| None`, `min_length=5, max_length=5` | 5-digit FIPS or `None` | string/optional |
| `extraction_intensity` | TERRITORY | plain `float` with `Field(ge=0.0, le=1.0)` — **not** the shared `Intensity` type alias despite an identical `[0,1]` domain (contrast `models/types.py:130-137`'s `Intensity` vs `models/entities/territory.py:171-176`'s bare `float`) | `[0.0, 1.0]` | unit-interval, write-only here |
| `la_production` | graph-level (`G.graph[...]`), not a node/edge attribute | plain Python `dict[str, float]` — no Pydantic type at all | keys = worker node ids, values `≥ 0` | **out-of-model scratch state** |
| `base_year` | graph-level | plain `int` | unconstrained | out-of-model scratch state, read with **two different Python-side defaults** across call sites (`2022` here vs `2010` at `tick/system/__init__.py:420` and `kernel/sim_clock.py:19`) — moot in practice since it lives on the dead branch |
| `economy.base_labor_power` (define) | — | `float`, `ge=0.0` | `[0.0, ∞)` | unbounded real coefficient |
| `timescale.weeks_per_year` (define) | — | `int`, `ge=1` | `[1, ∞)` | integer coefficient |

**Enum discriminant flag.** `role` (`SocialRole`, 8 members) needs the same `defenum`/enum-`deffield`
treatment ADR195/196 already landed for `organization/kind` (`OrgKind`, 4 members) — direct precedent,
not a new gap (§6). Only two of the eight members (`PERIPHERY_PROLETARIAT`, `LABOR_ARISTOCRACY`) are
ever compared against in this system.

**Boolean flag, with a direct same-node-type precedent.** `active` needs BSL's established bool→int
0/1 workaround: `deffield` has **no `bool` storage type at all**, despite `bool` parsing as a legal
`:type` token in other typechecker contexts (`declarations.rs:649`) — confirmed verbatim by a landed
content pack's own comment: *"0/1 rather than #t/#f: BSL has Bool (§3.1) but `deffield` has no bool
type and `GraphSubstrate` attributes are f64. Recorded, not hidden."*
(`content/scenarios/vitality-conformance.bscn:19-20`, `(deffield social-class/active int
extensive)`) — the **same** `SOCIAL_CLASS` node type this system also reads `active` off of. This
**corrects** one item in the task's "CURRENT BSL surface" notes, which listed `bool` as part of
`deffield`'s type vocabulary: `bool` is not a usable `deffield` storage type in practice; the
established idiom is `int extensive` with `0`/`1` values.

**Currency flag — same conclusion as Territory/Metabolism, not a hard stop.** `wealth` is
`Currency`-typed in the Python reference but `:field`-sourced reads always evaluate as `Value::Real`
in BSL, never `Value::Currency` (Territory's own finding, confirmed still true: `scenario.rs:1067`
refuses `Currency`-typed field **storage** outright — `BslType::Currency => Err(...)`, and every
landed pack routes around it by declaring money-like fields `int extensive`, e.g.
`vitality-conformance.bscn:22`'s `(deffield social-class/wealth int extensive)`). Since none of this
system's multiplies involve a genuinely `Value::Currency`-typed operand (both `wealth` and
`produced_value` are plain reals at evaluation time), the D-1-class `Currency × Ratio` scale-op
hazard Territory/Metabolism hit **does not arise anywhere in this system's own arithmetic** —
simpler than Territory's `rent_level × rent_spike_multiplier` case.

**Out-of-model scratch state flag — the system's most significant type-inventory finding.**
`la_production` has no Pydantic type, no node home, no edge home — it is a raw Python dict living
only in `G.graph[...]`, read back by a *different* system (`economic.py:438`, §5). This is the exact
shape `bsl-language.rst` §3.6 names "Q6: the single most pervasive gap in the estate" (§6).

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `float`/`int`). Grep-confirmed zero `exp`/`log`/`sigmoid`/
`pow` calls anywhere in production.py — **this system has zero libm-nondeterminism hazard**, matching
Territory and Solidarity, unlike Metabolism. Shapes, in execution order:

1. **Division (annualization):** `annual_labor_power / weeks_per_year` (production.py:114) — Real ÷
   Int. No bare literal (both operands are runtime-resolved define values).
2. **Floor division + addition (dead branch):** `hydrated_base_year + tick // weeks_per_year`
   (production.py:167) — inside the provably-unreachable tensor branch (§2). Not load-bearing for
   the port; recorded for completeness since port-as-is law would otherwise require transcribing it.
3. **Conditional division with zero-guard:** `0.0 if max_biocapacity <= 0 else biocapacity /
   max_biocapacity` (production.py:155) — one comparison, one division, one bare literal (`0.0`, the
   ternary's "true" branch result) needing the "no bare non-integer literal" scaled-lit/Real-zero-
   promotion treatment Territory and Solidarity both already flagged.
4. **Division (dead branch):** `tensor.total_v / weeks_per_year` (production.py:172) — inside the
   same unreachable branch as item 2; `tensor.total_v` is itself a Pydantic `@computed_field` sum of
   four `DepartmentRow.v` fields (`domain/economics/tensor.py:363-374`), never evaluated in practice.
5. **Two multiplies (extensive scaling):** `(effective_labor_power * population) * bio_ratio`
   (production.py:175) — an intensive per-capita rate (`effective_labor_power`) times an extensive
   population count times an intensive ratio (`bio_ratio`), yielding an extensive `produced_value`.
   This is the **correct** extensive/intensive handling the "intensive-aggregation-variance-error"
   footgun exists to catch — population-scaling happens before any cross-node summation, never an
   unweighted mean of a per-head rate across classes.
6. **Additive accumulation, direct producer:** `current_wealth + produced_value`
   (production.py:181) — plain add, self-write.
7. **Additive accumulation, employed producer (structural note, not a math hazard):**
   `employer_wealth + produced_value` (production.py:192) — plain add, but a **sequential
   read-modify-write across loop iterations on a node OTHER than the one being iterated**: if two or
   more `LABOR_ARISTOCRACY` workers share one employer, each iteration's `graph.get_node(employer_id)`
   re-reads the PRIOR iteration's already-applied write. Mathematically the sum is order-independent
   (addition is associative/commutative up to float rounding), but the push-style, iteration-order-
   dependent mutation pattern does not match BSL's `for-each` semantics, where every effect in one
   `for-each` reads the SAME pre-state (confirmed: `evaluator.rs:112`, "matching §2.8's own worked
   `for-each` example, whose `emit` reads the PRE-scale …"). A naive one-rule-per-worker translation
   would have each worker's write silently overwrite rather than accumulate when workers share an
   employer — see §6 for the pull-style-fold reformulation this needs.
8. **Additive accumulation, per-territory (favorable, order-independent by construction):**
   `territory_production.get(territory_id, 0.0) + produced_value` (production.py:202-204) — a
   running dict, collect-as-you-go but never re-read by a LATER worker's routing decision (only
   consumed after the whole loop, in Computation 4) — the same "genuinely favorable structural match"
   to BSL's per-position pre-state semantics Territory's own §4.6 finding noted for its spillover
   accumulation.
9. **Conditional division with zero-guard + upper-only clamp:** `min(1.0, total_production /
   max_biocapacity) if max_biocapacity > 0 else 0.0` (production.py:267) — one comparison, one
   division, one `min`, two bare literals (`1.0`, `0.0`). **No scalar `min`/`max` form exists in
   BSL's `<expr>` grammar** (`<arith> ::= "+" | "-" | "*" | "/"`, bsl-language.rst:1178) — transcribes
   as a nested `if`, the same idiom Territory's Phase-3 spillover and Solidarity's single clamp both
   already established as portable. Because `total_production ≥ 0` (only ever accumulated from
   `produced_value > 0` values, production.py:201) and the branch already guards
   `max_biocapacity > 0`, the ratio can never be negative — **no lower-bound clamp is mathematically
   needed here**, unlike Territory's `heat`, so this is not an inconsistency bug, just a different
   (independently correct) hand-rolled style from the shared `SystemBase._write_clamped` helper
   (`system_base.py:161-191`) that Territory Phase-1/Metabolism's `habitability` both use instead.
10. **Real→Int demotions: none.** Grep-confirmed zero `int(...)` casts anywhere in production.py — a
    favorable contrast with Territory's two `floor`-class demotions (population displacement, camp
    decay); `population` here is read as an already-integer attribute, never cast.
11. **Currency-mixing multiplies: none**, as established in §3 — every operand in this system's
    arithmetic is `Value::Real` at BSL evaluation time.

**Net float-op assessment:** no libm, no Int demotion, no Currency-scale hazard, one correctly-
guarded clamp with no cross-site inconsistency. The only structural math-adjacent hazard is item 7's
push-vs-pull accumulation shape (§6), which is a reformulation question, not an arithmetic one.

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 3.0** (production.py:68), confirmed against `_SYSTEM_CLASSES`
  (`simulation_engine.py:328-363`): `Vitality(1.0) → Territory(2.0) → Substrate(2.5) →
  Production(3.0) → TickDynamics(4.0) → ReserveArmy(5.0) → … → Solidarity(8.0) → ImperialRent(9.0) →
  … → Metabolism(13.0) → …`.
- **Reads from a same-tick prior system — a genuine channel, unlike Territory's "none":**
  - `SOCIAL_CLASS.active`, `.population`, `.wealth` — all three written by `VitalitySystem` (@1.0)
    the SAME tick, before Production runs: `wealth` drained by the subsistence burn
    (`vitality.py:116-122`), `population` reduced by attrition (`vitality.py:130-132`), `active`
    flipped to `False` on death (`vitality.py:169-174`). A worker that dies at position 1.0 is
    correctly excluded from production at position 3.0 the same tick.
  - `TERRITORY.biocapacity`/`.max_biocapacity` — **NOT** written same-tick before Production reads
    them. `SubstrateSystem` (@2.5, immediately prior) explicitly does **not** touch these fields
    (module docstring, `substrate.py:31`: "Does NOT touch `Territory.biocapacity`/
    `MetabolismSystem`"); the only writer, `MetabolismSystem`, runs at position 13.0 — strictly
    AFTER Production this same tick. So the `bio_ratio` this system computes always reflects the
    PREVIOUS tick's committed biocapacity (or the scenario seed on tick 0), never a same-tick write.
- **Writes consumed later this tick / downstream ticks:**
  - `TERRITORY.extraction_intensity` — read same-tick, immediately downstream, by `MetabolismSystem`
    (@13.0, `metabolism.py:98,109`, feeding `calculate_biocapacity_delta`,
    `formulas/metabolic_rift.py:12,47`) — closing a tight cross-tick loop: Production's
    `extraction_intensity` write depletes NEXT tick's `biocapacity`, which THIS tick's `bio_ratio`
    read used the PREVIOUS tick's value of. Also read, explicitly one-tick-lagged BY DESIGN, by
    `SubstrateSystem` (@2.5, `substrate.py:39-43,151-154,229`: *"this system reads each territory's
    `extraction_intensity` as it stands at the START of this tick — i.e. whatever ProductionSystem
    (@3.0) wrote during the PREVIOUS tick"*).
  - `SOCIAL_CLASS.wealth` — one of eight systems that write this attribute
    (`tools/regression_scenarios.py:2880-2890`'s `CHANNEL_WRITERS["wealth"]`:  `VitalitySystem`,
    **`ProductionSystem`**, `ImperialRentSystem`, `StruggleSystem`, `MarketScissorsSystem`,
    `DispossessionEventSystem`, `DecompositionSystem`, `OODASystem`) and read downstream by at least:
    `ImperialRentSystem` (@9.0, `economic.py:228,283,296,375,386,466,499,514,578,590`),
    `SurvivalSystem` (@15.0, `survival.py:127`), `StruggleSystem` (@16.0, `struggle.py:380,501`),
    `ideology`/`ConsciousnessSystem` (@17.0, `ideology.py:313`), `AllegianceSystem` (@17.42,
    `allegiance.py:289,436`), `ContradictionSystem` (@18.0, `contradiction.py:528`),
    `ContradictionFieldSystem` (@19.0, `contradiction_field.py:115,150`) — one of the most
    heavily-shared attributes in the graph.
  - `la_production` (graph-level) — read **only** by `ImperialRentSystem`
    (`economic.py:438,453`: `la_production = graph.get_graph_attr("la_production", {}); …
    productivity_value = la_production.get(edge.target_id, 0.0)`), the SAME tick, three positions
    later (@9.0). This is the channel §6 finds blocked.
- **Context/service usage with no BSL equivalent:** `services.tensor_registry` (a `getattr` with
  `None` default, declared on `ServicesProtocol`, `kernel/services.py:55`) and
  `graph.get_graph_attr("base_year", 2022)` — both confined to the provably-dead branch (§2), so
  neither is a live cross-system input in practice. `self.invariants`/`self.phase`/`creates_value`
  (Spec-040 declarations, production.py:73-87) are self-tested only
  (`tests/unit/engine/test_production_spec040.py`) — `creates_value` IS consumed, but only by the
  property-test conservation registry (`tests/property/harness/system_registry.py:7`), never by the
  tick engine itself; `invariants`/`phase` have no consumer anywhere in `src/` or `tests/` beyond
  their own declaration test. None of these three carries tick-affecting behavior, so none is a
  porting concern — noted for completeness only.
- **DORMANCY on canonical scenarios — this system is LIVE, not dormant, unlike Territory/Solidarity.**
  `tools/regression_scenarios.py`'s `SCENARIO_COVERAGE_DATA` for `imperial_circuit` explicitly claims
  and (per the module's own verification discipline, lines 300-308: rows are "verified via a live
  spot-run… not invented from source reading alone") confirms: `{"system": "ProductionSystem", "kind":
  "entity_delta", "key": "C003.wealth", "claim": "labor-aristocracy production routes its value to
  the employing core bourgeoisie's wealth (Amin/Wallerstein employed-producer routing)"}`
  (regression_scenarios.py:321-326). `C003` is `CORE_BOURGEOISIE_ID`
  (`models/entity_registry.py:35`) — the employer in `create_imperial_circuit_scenario`
  (`engine/scenarios/_legacy.py:343-358` seeds `CORE_BOURGEOISIE`/`LABOR_ARISTOCRACY` with a `WAGES`
  edge and `TENANCY` edges, `_legacy.py:430,476-490`). So Computations 2's employed-producer routing
  (the `WAGES`-edge lookup, cross-node wealth write, `la_production` recording) is confirmed LIVE on
  the flagship canonical scenario — a materially different dormancy profile than Territory's
  ADJACENCY-gated phases or Solidarity's edge-lane block. **UNVERIFIED, narrower gap:** whether the
  "no employer found" fallback (production.py:195-198) is ever exercised on any canonical
  scenario — search run: `grep -n "LABOR_ARISTOCRACY" engine/scenarios/_legacy.py` shows every LA
  seed paired with a `WAGES` edge (`_legacy.py:358` + `:430`/`:490`), so this fallback branch is
  likely dead-by-construction on `imperial_circuit`; not independently confirmed against every other
  `SCENARIOS` entry (`two_node`, `single_county`, the electoral-golden scenarios) within this
  session's scope.
- **The `fips_code`/tensor-registry branch (§2) is dead on EVERY scenario**, including
  `single_county`, the one purpose-built to exercise a real hydrated `TensorRegistry` — a stronger
  and more specific dormancy finding than a mere "no canonical scenario seeds X" gap, since here the
  scenario-author's intent (per `single_county.py`'s own docstring) is thwarted by a field-name
  mismatch inside this system itself, not by an absent seed.

## 6. BLOCKER ASSESSMENT

Adjudicated against the current dev tree (`rust/crates/babylon-bsl/src/{declarations,scenario,
evaluator}.rs`; `docs/reference/bsl-language.rst` §2.4/§2.7/§3.6), verified live in this session
(§1's citation list). Per the task's CURRENT BSL surface notes, Query Lane Slice 1 (`fold`,
`neighbors`, `select-max`/`select-min`, `field-of`, `for-each`, `exists`/`forall`, `update-node`
against a computed `NodeRef`) is LANDED and evaluates through `run_once_into` — verified directly
(`eval_fold`/`eval_selection` present in `evaluator.rs:556-825`).

| Computation | Verdict | Detail |
|---|---|---|
| Computation 1 — annualization (`production.py:114`) | **PORTABLE NOW** | Plain `Real / Int` division; `<arith>` grammar includes `/` (bsl-language.rst:1178). Same "annual → weekly" idiom already established in Territory/Metabolism/Vitality packs — a `defconst` ratio, no new precedent needed. |
| Tensor-registry `effective_labor_power` branch (`production.py:160-172`) | **NOT-A-PACK — verified dead, D-record the omission** | Provably unreachable on every scenario in the estate (§2, §5), not merely "provably uniform" (Territory's `displacement_mode` precedent) — there is no live value to even declare `:const`. Recommend: omit this branch from the pack entirely, D-record citing this finding (fips_code/county_fips mismatch) rather than transcribing dead code that could never fire. If port-as-is discipline instead requires transcribing it verbatim as unreachable dead code, the pieces themselves (`field-of` a territory's would-be fips field, a `:metric`-style registry lookup) have no BSL equivalent for an external keyed cache lookup (`TensorRegistry`) at all — moot either way. |
| `active`/`role`/`population` gating + tenancy lookup (`production.py:135-146`) | **PORTABLE WITH D-RECORD** | `active`: bool→int 0/1 workaround, direct SAME-node-type precedent (`vitality-conformance.bscn:19-22`). `role`: enum-`deffield` workaround, direct precedent (`organization-foundation.bscn:44-45`, ADR195/196, ORG_KIND 4-member enum → `SocialRole` 8-member enum, same mechanism). Tenancy lookup: single-target typed-neighbor query (`neighbors`/`the`), Query Lane Slice 1 landed and ADR197-proved end to end for Territory's own sink-selection (`_find_sink_node` precedent). All three pieces individually proven on dev; no landed pack yet composes exactly this three-piece shape for THIS system, hence D-record not full confidence. |
| `bio_ratio` + territory field reads (`production.py:149-155`) | **PORTABLE NOW** | `field-of` over a `NodeRef` (Slice 1 landed) for `biocapacity`/`max_biocapacity`; zero-guarded division via `if`/`<=`/`/` — all in the landed `<cond>`/`<expr>` grammar (bsl-language.rst:741-749,1170-1188). |
| `produced_value` computation (`production.py:175`) | **PORTABLE NOW** | Two plain `Real * Real` multiplies, no Currency-scale hazard (§3). |
| Direct-producer wealth write (`production.py:181`) | **PORTABLE NOW** | Self-node `update-node`, plain `Real + Real` add. `wealth` declared `int extensive` per the established money-field workaround (§3), same ADR183 declared-deviation class as Territory's `rent_level`. |
| Employed-producer employer lookup + cross-node wealth write (`production.py:185-194`) | **PORTABLE WITH D-RECORD (nontrivial reformulation) — UNVERIFIED IN PRACTICE** | Employer lookup: single-target typed-neighbor query over `WAGES` (same mechanism as tenancy lookup, above). `update-node` against a computed `NodeRef` (the employer) is the exact ADR197-proved shape (Territory's population-transfer precedent). **But** the frozen Python's push-style sequential read-modify-write (§4 item 7) does not match `for-each`'s shared-pre-state semantics (`evaluator.rs:112`) — when 2+ LA workers share one employer, a naive per-worker `update-node` translation would overwrite rather than sum. The mathematically-identical fix is a pull-style grouped `fold sum` (per employer, sum the production of its incident LA employees, itself requiring each employee's own nested territory lookup inside the fold body). `bsl-language.rst` §2.7's `<fold>` grammar is recursive over `<expr>` and its own worked example nests a `fold sum` inside a `fold sum` with an `:as`-named outer element reaching two hops deep (`(fold sum (hyperedges …) :as sector (fold sum (members-of sector …) (field-of it …)))`, bsl-language.rst:1149-1151) — grammatically this shape is provided for, but **no landed pack anywhere in `rust/crates/babylon-tick/content/` exercises a fold this deep for a class→tenancy-territory→employer three-hop shape**. Flag as unverified-in-practice, not a hard blocker. |
| No-employer fallback (`production.py:195-198`) | **PORTABLE WITH D-RECORD, likely-dead branch** | Mechanically identical to the direct-producer write (a self-node `update-node`); transcribe port-as-is per the same discipline as the tensor branch, though this one is NOT provably dead (only likely-dead by construction on `imperial_circuit`, unverified on the full `SCENARIOS` set — §5). |
| Per-territory production accumulation (`production.py:200-204`) | **PORTABLE NOW** | Order-independent running-dict sum, favorable structural match to BSL per-position semantics (§4 item 8) — reformulates cleanly as a `fold sum` over `TENANCY`-incident producers grouped by territory, the SAME reformulation family as the employer case but one hop shallower (no employer indirection). |
| `la_production` graph-level publish (`production.py:207`) | **BLOCKED — named lane: graph-level scratch-state storage (no `GraphSubstrate` construct)** | `GraphSubstrate`/BSL attribute storage is strictly per-node or per-edge; there is no graph-scope scratch dict. Confirmed by grep: zero `graph_attr`/graph-level-metadata construct anywhere in `rust/crates/babylon-graph/src/` or `rust/crates/babylon-bsl/src/`. `bsl-language.rst` §3.6 names this exact gap directly: *"Twenty-two of the thirty-four frozen systems read or write state that belongs to the graph rather than to any node… (R9 gap analysis §2, Q6: the single most pervasive gap in the estate)"* and records a **"[draft ruling — Phase 1 review, R9 chapter C3]"** fix — represent graph-scope state as an ordinary `deffield` on a declared CARRIER `NodeType` with `:ceiling 1`, read via `(field-of (the NodeType/…) …)`, written via `(update-node (the NodeType/…) …)`. This is a real, on-paper path, but explicitly **"not a settled law"** (bsl-language.rst:106: "each row is a Phase-1 review item, not a settled law") and **zero landed content packs use it** — grep-confirmed zero `:ceiling 1` occurrences anywhere under `rust/crates/babylon-tick/content/`. Name the exact missing lane precisely: **carrier-NodeType graph-scope state (bsl-language.rst §3.6, R9 chapter C3) — documented, unimplemented-in-practice.** Until either that pattern is proven on a landed pack or a real graph-attribute construct lands, this computation cannot be ported to a working conformance vector. |
| Extraction-intensity broadcast (`production.py:246-268`) | **PORTABLE NOW, reformulated as a fold** | Reformulates as a per-territory `fold sum` over `TENANCY`-incident active producer nodes' `produced_value` (same shallow reformulation as the per-territory accumulation row above — indeed the same underlying quantity), then the zero-guarded-division-plus-upper-clamp idiom already established as portable (Territory Phase-3, Solidarity's single clamp). No Currency hazard, no libm, `min` expressed as nested `if` per the no-scalar-min/max grammar constraint. |

**RESERVED-LINE note.** `ProductionSystem` contains no doctrine-tree content, no explicit National
Question parameter, and no outcome-definition logic — its numeric coefficients
(`base_labor_power`, `weeks_per_year`) are engineering constants, not ideologically-authored values.
**However**, the system's producer-role ROUTING STRUCTURE — which `SocialRole`s produce directly
versus route through an employer, i.e. the Direct-Producer/Employed-Producer split between
`PERIPHERY_PROLETARIAT` and `LABOR_ARISTOCRACY` — is the direct engine mechanism materializing the
Amin/Wallerstein imperial-bribe model that underlies the project's MLM-TW National Question line
(ADR171, `ai/national-question-ruled-2026-07-28.md` per project memory). This inventory describes
that structure faithfully (§2 Computation 2) and does not propose changing it; any future port
decision that would alter WHICH roles are direct vs. employed producers, or introduce a third
routing class, should be treated as touching the reserved ideological line and escalated to the
Director rather than decided as an ordinary port-as-is transcription call.

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_production.py` | 490 | **Primary conformance-oracle candidate.** `TestProductionSystem` (basic wealth generation, biocapacity-ratio scaling, inactive/non-producer skip, no-tenancy skip), `TestProductionPopulationScaling` (Mass Line population multiplier), `TestTensorAwareProduction` (Feature 020 tensor-registry tests — these are the tests that hand-stamp `fips_code` directly via `graph.add_node`/`.nodes[...][...]=`, bypassing `Territory`; they pass, but exercise a branch verified dead on every real scenario, §2). Direct helper functions (`_create_worker_node`, `_create_territory_node`, `_create_tenancy_edge`) build graph nodes by raw kwargs, not via the `Territory`/`SocialClass` Pydantic models — useful transcription reference for BSL field names, but not evidence the tensor branch is reachable in production. |
| `tests/unit/engine/systems/test_production_extraction_intensity.py` | 378 | **Primary conformance-oracle candidate for Computation 4 and the cross-tick Metabolism loop.** `TestProductionSetsExtractionIntensity` (linkage correctness, the `min(1.0, total_production/max_biocapacity)` formula, multi-worker-per-territory summation), `TestExtractionIntensityCausesDepletion` (drives `MetabolismSystem` after `ProductionSystem` to confirm the ΔB = R − (E·η) hump-shape dynamic the module docstring documents mathematically). |
| `tests/integration/engine/systems/test_production_extraction_intensity.py` | 169 | Long-run integration variant ("Extracted from" the unit file above) — many-tick decay-phase dynamics for the same Production→Metabolism loop; a candidate multi-tick conformance vector, not unit-scoped. |
| `tests/unit/engine/test_production_spec040.py` | 30 | Schema-level only: pins `ProductionSystem.invariants`/`.phase` declarations (Spec-040 discipline). Not a behavioral/conformance oracle — these attributes have no engine-side consumer (§5). |

**Namespace collision, noted so it is not mistaken for ProductionSystem coverage:**
`tests/unit/economics/substrate/test_production.py` (211 lines) tests `DefaultHexProductionComputer`
(`domain/economics/substrate/production.py`) — an unrelated hex-grid Volume-I production computer,
not this system. Likewise `tests/unit/economics/tensor_hierarchy/{,leontief_rent/}test_production_chain_rent.py`
and the various `test_reproduction.py` files test unrelated Leontief/reproduction-schema modules.
Excluded from the table above.

**qa:regression byte-gate coverage.** `tools/regression_test.py::graph_content_hash` (lines 924-964)
hashes every node/edge attribute of the `WorldState→graph` projection on every canonical scenario —
so `SOCIAL_CLASS.wealth` and `TERRITORY.extraction_intensity` (both node attributes) ARE covered by
the byte-identical gate, and §5 confirms `imperial_circuit` genuinely exercises both the direct- and
employed-producer paths. **But `graph_content_hash` explicitly excludes `g.graph` metadata**
(docstring, regression_test.py:939-943: *"Graph metadata (`g.graph`: economy, event log, opposition
states) is also excluded"*) — and `la_production` lives exactly there. **The `la_production`
cross-system channel (§5, §6) is therefore invisible to the qa:regression byte-gate entirely**: a
regression in how LA production is ledgered and paid back by `ImperialRentSystem` would not be caught
by the byte-identity gate, only (if at all) by `ImperialRentSystem`'s own downstream wealth-delta
checks. Worth flagging for the eventual pack's conformance-fixture design — this channel needs its
own explicit assertion, not a free ride on the graph hash.

---

## Adjudication (2026-08-12)

Adjudicated against the **current dev tree** (`9324482f`) by a second, read-only
pass, in the manner of the Territory inventory's own "Adjudicated verdict"
section. Two corrections, six confirmations.

1. **CONFIRMATION, and credit — the report's `bool` correction to the task
   brief is right, and it generalizes across this batch.**
   `rust/crates/babylon-bsl/src/scenario.rs:919-931` (`load_deffield`) admits
   exactly `int / probability / intensity / coefficient / currency` plus the
   `enum` branch at `:890`; `bool` reaches only `declarations.rs:649`
   (`parse_type_name`), which no `.bscn` field declaration routes through —
   `scenario.rs:1071-1078`'s own comment states the admitted set. The landed
   precedent the report cites is verbatim:
   `content/scenarios/vitality-conformance.bscn:19-22`. This finding also
   invalidates the SubstrateSystem inventory's proposed `bool` eligibility
   discriminator (adjudicated there as correction 1) — worth carrying into the
   #536 design gate as a batch-level fact, not a per-system one.
2. **CORRECTION — D102 blocks both fold reformulations this report rates
   portable, and the fix contradicts the report's own `role` D-record.**
   `field-of` naming an `:enum-type`-declared field is **REFUSED AT LOAD**:
   `rust/crates/babylon-bsl/src/typecheck.rs:266-289`
   (`check_no_field_of_on_enum_field`) — *"field-of {qname}: an
   :enum-type-declared field is read via a :field binding only (§2.5) —
   field-of is not extended to enum-declared fields (§2.13, D102)"* (no error
   code minted; `:804-805` pins it). The landed fold idiom reads element fields
   as exactly `(field-of it <qname>)`
   (`rust/crates/babylon-tick/tests/query_lane_e2e.rs:152-153, 238-239`), i.e.
   off a NON-subject node. Both of this report's reformulations do that with
   `role`:
   - the per-territory extraction fold (§6 last row, rated **PORTABLE NOW**) —
     `(fold sum (neighbors self EdgeType/TENANCY :in NodeType/SOCIAL_CLASS) …)`
     must filter `role in _PRODUCER_ROLES` (production.py:141-143) per element;
   - the employed-producer employer fold (§6, rated PORTABLE WITH D-RECORD) —
     the same per-element `role` test over `WAGES`-incident employees.

   Neither is portable while `role` is a `defenum`/`:enum-type` field. The only
   route today is an **int-ordinal `role` encoding**, which directly contradicts
   §3's and §6's own recommendation to use the ADR195/196 enum-`deffield`
   precedent (`organization-foundation.bscn:44-45`). The two D-records are
   mutually exclusive and the pack must choose one; the extraction-broadcast row
   must move from **PORTABLE NOW** to **PORTABLE ONLY UNDER AN INT-ORDINAL
   `role` ENCODING**. (`active` is unaffected — it is already `int` 0/1 by
   correction 1's precedent, and `population`/`wealth`/`biocapacity` are
   ordinary numerics.)
3. **CORRECTION — the D116 same-anchor pre-state fact is never applied, and a
   Production pack is a multi-rule pack.** The task brief's CURRENT BSL surface
   names it ("TWO rules at one anchor position do NOT yet share pre-state");
   this report's §6 never reaches it, even though the frozen system is three
   distinct iterations (direct-producer self-write, employed-producer employer
   accumulation, per-territory broadcast) whose *ordering* is behaviour-relevant
   in the Python. Adjudicated: the pack is **safe on the substance** — the
   extraction fold recomputes `produced_value` from pre-tick
   `biocapacity`/`population` rather than reading Computation 2's `wealth`
   writes (production.py:265-267 reads only `max_biocapacity` and the
   in-memory accumulator), and no node can be both a direct producer and an
   employer since `_DIRECT_PRODUCER_ROLES` and `_EMPLOYED_PRODUCER_ROLES` are
   disjoint (production.py:46-52) — but the blocker table owes that analysis as
   an explicit row rather than an omission, because it is what licenses the
   multi-rule decomposition at all.
4. **CONFIRMATION — the `fips_code` defect, independently reproduced end to
   end.** `src/babylon/engine/systems/production.py:164` reads
   `territory_attrs.get("fips_code")`; `Territory` declares `county_fips` and no
   `fips_code` (`src/babylon/models/entities/territory.py:81-91`, sole `fips`
   hits at `:38,81,89`); `WorldState.to_graph` stamps territory nodes via
   `territory.model_dump()` (`src/babylon/models/world_state.py:746`), so no
   live territory node can carry the key; `resolve_county_identity`'s docstring
   is the ground truth the report quotes —
   `src/babylon/domain/economics/tick/graph_bridge.py:46-47`, *"The county
   identity of a territory lives in its `county_fips` attribute and nowhere
   else."* Repo-wide, `fips_code` on a territory graph node appears only in
   hand-stamped unit fixtures; the two `tools/regression_test.py:244,255` hits
   are the tensor FIXTURE's own dict key, not a node attribute — which
   independently confirms the report's sharpest point, that `single_county`
   hydrates a real registry the branch still cannot reach. **Verified
   load-bearing defect; port-as-is disposition (omit + D-record) is right.**
5. **CONFIRMATION, with added teeth — the `la_production` blocker is harder than
   "documented, unimplemented-in-practice".** Zero graph-level attribute
   construct exists in either Rust crate (`rg -n
   "graph_attr|graph_attribute|GraphAttr" rust/crates/babylon-graph/src/
   rust/crates/babylon-bsl/src/` → 0 hits) — the report's grep reproduces
   exactly. Additionally the R9 chapter-C3 carrier-NodeType pattern this row
   names as the on-paper path is **itself unserved**: C3 prescribes
   `(field-of (the NodeType/…) …)` / `(update-node (the NodeType/…) …)`
   (`docs/reference/bsl-language.rst:2664-2668`) and `the` is in
   `UNSERVED_EXPRESSION_HEADS` under **slice 2**
   (`rust/crates/babylon-bsl/src/evaluator.rs:504-506`). So the gap is not "a
   real on-paper path nobody has exercised" — the path's own accessor does not
   evaluate. **BLOCKED stands, reinforced.**
6. **CONFIRMATION — the byte-gate blind spot is exactly as claimed.**
   `tools/regression_test.py:941-943` (`graph_content_hash` docstring): *"Graph
   *metadata* (`g.graph`: economy, event log, opposition states) is also
   excluded, because the spec's field set is nodes/edges/actions."*
   `la_production` is written via `graph.set_graph_attr` (production.py:207) and
   read via `graph.get_graph_attr` (`src/babylon/engine/systems/economic.py:438,453`)
   — the only two sites repo-wide. The channel is invisible to the byte gate.
   Confirmed.
7. **CONFIRMATION — the cross-system channel table's two most load-bearing
   writes spot-check clean.** (a) `TERRITORY.extraction_intensity` → read by
   `src/babylon/engine/systems/metabolism.py:98,109` (same tick, @13.0) and by
   `substrate.py:229` (one-tick lag, @2.5) — both confirmed by direct read. (b)
   `SOCIAL_CLASS.wealth` → `tools/regression_scenarios.py:2880-2889`'s
   `CHANNEL_WRITERS["wealth"]` lists exactly the eight writers the report names,
   `ProductionSystem` among them; the header comment (`:2870-2878`) records that
   this row was itself corrected against source. Confirmed.
8. **CONFIRMATION — tick position 3.0, and the reformulation's direction
   operand exists.** `src/babylon/engine/systems/production.py:68`
   (`position: ClassVar[float] = 3.0`) against the 34-member `_SYSTEM_CLASSES`
   (`src/babylon/engine/simulation_engine.py:328-363`), order derived by sorting
   on `position` (`:376-377`). Separately confirmed for the fold reformulations:
   `neighbors` takes a mandatory direction operand
   (`(neighbors <expr> <EdgeType> <direction> <NodeType>)`,
   `rust/crates/babylon-bsl/src/query.rs:162-177`, `:in`/`:out`/`:any`), so
   pulling `TENANCY`-incident producers from a territory (`:in`) and
   `WAGES`-incident employees from an employer are both directionally
   expressible — the blocker is D102 (correction 2), not the query shape.

**FINAL VERDICT: PORTABLE WITH D-RECORDS for most computations — CONFIRMED, with
the portable set narrowed by D102 and the one BLOCKED channel hardened.** The
`fips_code`/`county_fips` defect is verified load-bearing and provably dead on
every scenario including `single_county`; zero libm hazards confirmed; the
`la_production` graph-level channel is BLOCKED and worse than described (the
carrier-NodeType route's own accessor `the` is slice-2 unserved). But the
**extraction-intensity broadcast is NOT portable now** and the employer-routing
fold is not portable under this report's own `role` enum-`deffield` D-record:
both read `role` off a non-subject node inside a fold body, which `field-of`
refuses at load for enum-declared fields (D102) — the pack must choose an
int-ordinal `role` encoding instead, and record that the two D-records are
mutually exclusive. RESERVED-LINE handling (Amin/Wallerstein producer-role
routing described, not touched) is correct and is the model the other four
inventories in this batch should have followed.
