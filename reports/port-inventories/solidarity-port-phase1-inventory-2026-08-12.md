# SolidaritySystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `SolidaritySystem` (`src/babylon/engine/systems/solidarity.py`, 203
lines, tick position 8.0) is a single per-edge loop over every `SOLIDARITY`-typed edge: it reads
the edge's own `solidarity_strength` attribute plus both endpoints' `class_consciousness`, computes
a transmission delta, clamps, and writes the target node's `ideology` dict. Every one of its five
computational steps is gated on first obtaining an `EdgeRef` and reading a field off it —
`solidarity_strength` lives **only** on the edge, never on either node — and on the current dev
tree (verified live against `rust/crates/babylon-bsl/src/evaluator.rs`) the `edges`/`edge-between`
accessors that would produce and read that `EdgeRef` are explicitly enumerated as **Slice 2, NOT
BUILT** (`UNSERVED_EXPRESSION_HEADS`, evaluator.rs:503-512). There is no `neighbors`-based
workaround: `neighbors` yields `NodeRef`s only (bsl-language.rst:1273), never the edge itself, so
the edge-resident datum is structurally unreachable without Slice 2. Zero libm hazards, two distinct
`EventType` emissions (both WS1 ledger rows, consumed only by the narrative layer today), and the
system is dormant (never fires `CONSCIOUSNESS_TRANSMISSION`/`MASS_AWAKENING`) on all five of the
*original* canonical `qa:regression` scenarios by declared, code-verified design — but two later
Program-25 electoral-golden scenarios (`debs`, `bernie_valve`) DO seed a nonzero `solidarity_strength`
edge, a fact the stale coverage-gap doc's "every canonical scenario" language does not capture.

**Verdict: BLOCKED — entirely, on the dyadic-edge query lane (BSL query-evaluation slice 2:
`edges`/`edge-between`/`field-of`-over-`EdgeRef`). Zero of the system's five computational steps are
portable today; the pure arithmetic itself carries no additional math-side blocker once slice 2
lands.**

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/solidarity.py` | 203 | **The target.** `SolidaritySystem`, one `step()` method: a single loop over `SOLIDARITY` edges. Read completely, line by line. |
| `src/babylon/formulas/solidarity.py` | 36 | `calculate_solidarity_transmission` — the transmission-delta formula the system calls via the formula registry (solidarity.py:114,152-157). |
| `src/babylon/kernel/node_access.py` | 38 | `class_consciousness_from_node` — shared accessor reading `ideology["class_consciousness"]` off a node payload (solidarity.py:33,140,148). Also used by `StruggleSystem`/`ImperialRentSystem`/`EpistemicHorizonSystem` (docstring, node_access.py:23; confirmed by grep, §5). |
| `src/babylon/config/defines/consciousness.py` | lines 12-44 (`SolidarityDefines`) | Coefficient source: `scaling_factor`, `activation_threshold`, `mass_awakening_threshold`, `negligible_transmission`, `superwage_impact`. Only 3 of the 5 fields are read by this system (§2e). |
| `src/babylon/data/defines.yaml` | `solidarity:` block, lines 182-187 | Player-editable coefficient values. |
| `src/babylon/config/defines/_assembler.py` | lines 242-249, 277-279 | Legacy flat-attribute aliases `SUPERWAGE_IMPACT`/`SOLIDARITY_SCALING`/`NEGLIGIBLE_TRANSMISSION` delegating to `SolidarityDefines`. `NEGLIGIBLE_TRANSMISSION` is the one solidarity.py actually calls (line 160); the other two are declared but have **zero call sites anywhere in the repo** (grep-confirmed) — dead legacy surface, not part of this system's live computation. |
| `src/babylon/engine/formula_registry.py` | line 118 | `registry.register("solidarity_transmission", formulas.calculate_solidarity_transmission)` — the registry indirection `services.formulas.get(...)` resolves through (solidarity.py:114). |
| `src/babylon/models/entities/social_class.py` | `IdeologicalProfile` lines 61-107; `SocialClass.ideology` field lines 335-338; `SocialClass.active` field lines 380-383 | Node model: the **nested** `ideology` sub-model (`class_consciousness`/`national_identity`/`agitation`) this system reads and rewrites wholesale, and the `active` liveness flag it gates on. |
| `src/babylon/models/entities/relationship.py` | `solidarity_strength` field, lines 116-119 | Edge model: `Coefficient` `[0.0, 1.0]`-domain field, the persistent edge-resident datum this system's entire computation is gated on. |
| `src/babylon/models/types.py` | `Coefficient`, lines 156-165 | `Annotated[float, Field(ge=0.0, le=1.0), SnapToGrid]` — quantized only at Pydantic instantiation, never mid-tick (same caveat the Territory inventory recorded for `Currency`/`Intensity`). |
| `src/babylon/models/graph.py` | `GraphNode` lines 91-145, `GraphEdge` lines 147-177 | Frozen Pydantic wire types `query_edges`/`get_node` return; `.attributes: dict[str, Any]` on both. |
| `src/babylon/kernel/graph_protocol.py` | `get_node` 77-86, `update_node` 88-98, `update_edge` 152-167, `query_edges` 278-296ish | `GraphProtocol` signatures the system calls (`query_edges`, `get_node`, `update_node`) — it never calls `update_edge` itself (only `CommunitySystem` does, §5). |
| `src/babylon/topology/graph.py` | `update_node` 660-670, `update_edge` 690-702 | Concrete `BabylonGraph` implementations — both are **plain dict merges with no type coercion or quantization** (`payload.update(attributes)`), identical in kind to Territory's finding. |
| `src/babylon/kernel/tick_partition.py` | `TickPartition`, lines 18-30 | `MATERIAL_BASE` — solidarity.py:90 declares this partition. |
| `src/babylon/kernel/system_protocol.py` | `ContextType`, line 16 | `type ContextType = "TickContext"` — the `context.get("tick", 0)` read at solidarity.py:172. |
| `src/babylon/kernel/system_base.py` | `SystemBase`, lines 58-117; `_publish` 194-196 | Abstract base. **Not exercised beyond the abstract `step()` contract** — `SolidaritySystem` calls `services.event_bus.publish(...)` directly (solidarity.py:173,191) rather than through `SystemBase._publish`; functionally identical (`_publish` is a one-line pass-through), not a defect. |
| `src/babylon/kernel/event_bus.py` | `Event` 33-56, `EventBus.publish` 134-152 | `Event(type: str, tick: int, payload: dict, timestamp: datetime)` frozen dataclass; `publish` appends to history then dispatches to handlers (fast path, no interceptors registered by default). |
| `src/babylon/models/enums/topology.py` | `EdgeType.SOLIDARITY = "solidarity"`, line 100 | The edge-type discriminant `query_edges(edge_type=EdgeType.SOLIDARITY)` filters on. |
| `src/babylon/models/enums/events.py` | `EventType.CONSCIOUSNESS_TRANSMISSION`/`MASS_AWAKENING`, lines 67-68 | The two emitted event types (§2's step 5, §5). |
| `src/babylon/engine/simulation_engine.py` | `_SYSTEM_CLASSES`, lines 328-363 | Confirms tick position 8.0 and full 34-system order (§5). |

**Not exercised by solidarity.py at all:** no `src/babylon/domain/*` module (the docstring at
solidarity.py:15-25 *documents* a theoretical correspondence to
`babylon.domain.dialectics.instances.connectivity.atomization_index` /
`AdjointCylinder.balance` but explicitly states "this system does not call it directly (no
behavior change in Phase B)" — a cross-reference for future theory work, not a live dependency;
not RESERVED-LINE, purely mathematical grounding text).

**Reference BSL/spec text read for the edge-accessor question** (all read in full for the cited
ranges): `docs/reference/bsl-language.rst` §2.10 "Element accessors" (lines 1805-1920) — whose own
worked examples for `edge-between`/`update-edge` are literally `EdgeType/SOLIDARITY`
(lines 1912, 2937); `rust/crates/babylon-bsl/src/evaluator.rs` lines 486-527
(`UNSERVED_EXPRESSION_HEADS`/`SERVED_QUERY_HEADS`) and 1185-1214 (`field-of` over `EdgeRef`
unreachable); `rust/crates/babylon-bsl/src/grammar.rs` lines 199-206, 638-647 and
`declarations.rs` lines 44-45 (confirming `edges`/`edge-between`/`the` **are** grammar-recognized
and arity/kind-checked — they parse and typecheck, they do not evaluate).

## 2. COMPUTATION CATALOG (execution order, `solidarity.py:97-202`, `step()` body)

### Step 1 — Edge selection + liveness/infrastructure gate (`solidarity.py:121-136`)
- **(a)** Iterate every `SOLIDARITY` edge in the graph; skip the pair if either endpoint is dead, and skip the edge entirely if it carries no built solidarity infrastructure (`solidarity_strength <= 0` — the "Fascist Bifurcation" design point, solidarity.py:10-13,83).
- **(b)** `for edge in graph.query_edges(edge_type=EdgeType.SOLIDARITY):` (solidarity.py:121) → `src_node = graph.get_node(edge.source_id)`, `tgt_node = graph.get_node(edge.target_id)` (123-124) → `if src_node and not src_node.attributes.get("active", True): continue` (127-128) → same for `tgt_node` (129-130) → `solidarity_strength = edge.attributes.get("solidarity_strength", 0.0)` (133) → `if solidarity_strength <= 0: continue` (135-136).
- **(c) Reads:** every `SOLIDARITY` edge's existence + its `solidarity_strength` attribute; both endpoints' `active` node attribute (default `True` if absent, defensive — a `None` node from a dangling edge reference also degrades safely to `{}` at line 139, never crashes).
- **(d) Writes:** none.
- **(e) Defines:** none consumed in this step.
- **(f) Events:** none.

### Step 2 — Source activation gate (`solidarity.py:138-144`)
- **(a)** Only a source class already in active revolutionary struggle (consciousness above the percolation threshold) can transmit.
- **(b)** `src_attrs = src_node.attributes if src_node else {}` (139) → `source_consciousness = class_consciousness_from_node(src_attrs)` (140, reads `ideology["class_consciousness"]`, node_access.py:29-37) → `if source_consciousness <= activation_threshold: continue` (143-144).
- **(c) Reads:** source node's `ideology.class_consciousness`.
- **(d) Writes:** none.
- **(e) Defines:** `solidarity.activation_threshold` (0.3, `[0.0, 1.0]`) — defines.yaml:184, `SolidarityDefines.activation_threshold` (consciousness.py:23-28). Fetched once at solidarity.py:117, reused in both this gate and the formula call.
- **(f) Events:** none.

### Step 3 — Transmission delta computation + negligible-delta gate (`solidarity.py:146-161`, `formulas/solidarity.py:10-36`)
- **(a)** The delta is proportional to both the built solidarity infrastructure and the consciousness gap between source and target — pure diffusion toward the source, never past it (see test_law_solidarity.py's L2, §7). Deltas too small to matter are dropped as noise (an `O(n²)` edge-saturation guard, defines.yaml:186).
- **(b)** `target_consciousness = class_consciousness_from_node(tgt_attrs)` (148, `old_consciousness = target_consciousness` captured at 149) → `delta = calculate_solidarity_transmission(source_consciousness, target_consciousness, solidarity_strength, activation_threshold)` (152-157) → inside the formula (formulas/solidarity.py:33-36): `if source_consciousness <= activation_threshold or solidarity_strength <= 0: return 0.0` (a **duplicate** of Step 1/2's guards, dead-in-practice since the caller already filtered both cases — a verbatim redundancy, not a defect, just double-gating) `else return solidarity_strength * (source_consciousness - target_consciousness)` → `if abs(delta) < services.defines.NEGLIGIBLE_TRANSMISSION: continue` (160-161).
- **(c) Reads:** target node's `ideology.class_consciousness`.
- **(d) Writes:** none (delta is a local value).
- **(e) Defines:** `solidarity.negligible_transmission` (0.01, `>= 0.0`, **no upper bound declared** — consciousness.py:35-39) via the `services.defines.NEGLIGIBLE_TRANSMISSION` legacy-alias property (`_assembler.py:277-279`).
- **(f) Events:** none.

### Step 4 — Clamp + node write (`solidarity.py:163-169`, helper `solidarity.py:45-75`)
- **(a)** Apply the delta to the target's consciousness, clamp to the valid unit interval, and write back the *entire* ideology profile (BSL-relevant: the write target is a single nested `ideology` dict attribute, not three independent scalar fields — §3 flags this).
- **(b)** `new_consciousness = target_consciousness + delta` (164) → `new_consciousness = max(0.0, min(1.0, new_consciousness))` (165) → `new_ideology = _update_ideology_class_consciousness(tgt_attrs, new_consciousness)` (168, defined 45-75: rebuilds `{"class_consciousness": new_consciousness, "national_identity": ideology.get("national_identity", 0.5), "agitation": ideology.get("agitation", 0.0)}` when `ideology` is already a dict, else synthesizes a fresh profile with the same defaults) → `graph.update_node(edge.target_id, ideology=new_ideology)` (169).
- **(c) Reads:** target's full `ideology` dict (for `national_identity`/`agitation` pass-through).
- **(d) Writes:** `SOCIAL_CLASS.ideology` (target only) — replaces the whole nested dict; `national_identity`/`agitation` are **defaulted to `0.5`/`0.0` if the pre-existing dict happens to be missing either key** (never actually observed in practice since `IdeologicalProfile` is a frozen model with defaults for both, but this is the literal behavior transcribed port-as-is).
- **(e) Defines:** none new (the clamp bounds `0.0`/`1.0` are bare literals, not defines — §4).
- **(f) Events:** none.

### Step 5 — Event emission (`solidarity.py:171-202`)
- **(a)** Every applied (non-negligible) transmission emits `CONSCIOUSNESS_TRANSMISSION` unconditionally; a second `MASS_AWAKENING` event fires only when this write crosses the target's consciousness up through the mass-awakening threshold.
- **(b)** `tick = context.get("tick", 0)` (172) → `services.event_bus.publish(Event(type=EventType.CONSCIOUSNESS_TRANSMISSION, tick=tick, payload={source_id, target_id, delta, solidarity_strength, source_consciousness, old_target_consciousness, new_target_consciousness}))` (173-187) → `if old_consciousness < mass_awakening_threshold <= new_consciousness:` (190, a Python **chained comparison** — two ANDed inequalities in one syntactic form, a mechanical BSL-transcription detail, not a semantic one) → `services.event_bus.publish(Event(type=EventType.MASS_AWAKENING, tick=tick, payload={target_id, old_consciousness, new_consciousness, triggering_source}))` (191-202).
- **(c) Reads:** none new.
- **(d) Writes:** none (event-bus history only, not graph state).
- **(e) Defines:** `solidarity.mass_awakening_threshold` (0.6, `[0.0, 1.0]`) — defines.yaml:185, `SolidarityDefines.mass_awakening_threshold` (consciousness.py:29-34).
- **(f) Events:** `EventType.CONSCIOUSNESS_TRANSMISSION` (always, on any applied transmission); `EventType.MASS_AWAKENING` (conditionally, on threshold crossing). **Two distinct `EventType` emissions total** — both consumed only by the narrative/AI-observer layer (`intelligence/ai/prompt_builder.py:248-255`, `intelligence/ai/director.py:94-118`, `game/chronicle_adapter.py:213-219`, `models/event_severity.py`, `projection/chronicle.py:172`) — confirmed by grep: **no `engine/systems/*.py` reads either EventType to branch its own logic.** Per the CURRENT BSL surface, `TickReport` carries no event log, so both are WS1 (#502) ledger rows, unpinnable by goldens today.

**Defines declared but never read by this system:** `solidarity.scaling_factor` (0.5, `[0.0, 2.0]`) and
`solidarity.superwage_impact` (1.0, `>= 0.0`) live in the same `SolidarityDefines` model but have
zero read sites inside `solidarity.py`. `scaling_factor` is cross-referenced only as a calibration
anchor in `OODADefines`' internal consistency table (`config/defines/ooda.py:401`,
`("embeddedness_discount", self.embeddedness_discount, s.scaling_factor)` — a documentation-style
assertion that two *different* systems' coefficients happen to match by design, not a runtime use
of `solidarity.scaling_factor` itself). `superwage_impact`'s only consumer is the dead
`_assembler.py:242-244` `SUPERWAGE_IMPACT` property, which itself has zero call sites anywhere in
the repo (grep-confirmed). Neither belongs in this system's port pack.

## 3. TYPE INVENTORY

Runtime storage note (identical finding to the Territory inventory, re-verified here):
`BabylonGraph.update_node`/`update_edge` (`topology/graph.py:660-670`, `:690-702`) are plain dict
merges with no type coercion or quantization. `Coefficient`'s `SnapToGrid` (1e-5 grid) and
`Probability`/`Intensity`'s equivalents apply only at Pydantic model instantiation, never mid-tick.
All in-tick arithmetic below is raw Python `float` with no grid quantization.

| Attribute | Node/edge type | Python model type | Domain | Category |
|---|---|---|---|---|
| `solidarity_strength` | edge, `SOLIDARITY` | `Coefficient` (`relationship.py:116-119`) | `[0.0, 1.0]` | unit-interval, **edge-resident** |
| `active` | node, `SOCIAL_CLASS` | `bool` (`social_class.py:380-383`, default `True`) | `{T, F}` | boolean latch, read-only here |
| `ideology` | node, `SOCIAL_CLASS` | `IdeologicalProfile` (nested `BaseModel`, `social_class.py:61-107`) | — | **composite/nested attribute** — see below |
| `ideology.class_consciousness` | (nested) | `float` `Annotated[..., ge=0.0, le=1.0]` (social_class.py:83-90) | `[0.0, 1.0]` | unit-interval, the only field this system actually computes |
| `ideology.national_identity` | (nested) | `float` `Annotated[..., ge=0.0, le=1.0]` (social_class.py:92-99) | `[0.0, 1.0]` | unit-interval, **read-and-passed-through unchanged** by this system, default `0.5` |
| `ideology.agitation` | (nested) | `float` `Annotated[..., ge=0.0]` (social_class.py:101-107) | `[0.0, ∞)` | **unbounded real**, **read-and-passed-through unchanged**, default `0.0` |
| `activation_threshold`, `mass_awakening_threshold` (defines) | — | `float` | `[0.0, 1.0]` | unit-interval coefficients |
| `negligible_transmission` (define) | — | `float` | `[0.0, ∞)`, **no upper bound** | unbounded real coefficient (consciousness.py:35-39 declares `ge=0.0` only) |
| `scaling_factor` (define, unused by this system) | — | `float` | `[0.0, 2.0]` | unit-interval-adjacent, dead in this system's scope |
| `superwage_impact` (define, unused by this system) | — | `float` | `[0.0, ∞)` | unbounded, dead in this system's scope |

**Composite-attribute flag — the genuinely new structural finding this system contributes (Territory
had none like it; every Territory attribute was a flat scalar).** `SocialClass.ideology` is a
`BaseModel`-typed field, and `WorldState.to_graph()` (`world_state.py:740-741`,
`G.add_node(entity_id, _node_type=NodeType.SOCIAL_CLASS, **entity.model_dump())`) recursively dumps
it, so the **runtime graph node's `ideology` attribute is a nested Python dict**
`{class_consciousness, national_identity, agitation}`, not three independent top-level keys.
`deffield`'s closed type vocabulary (`int, bool, currency, probability, intensity, coefficient,
enum`) is per-scalar-field only — there is no BSL representation of a nested payload as a single
declared field. Content-modeling workaround (same class of decision as Territory's enum flags, its
own D-record): declare three independent flat fields —
`social-class/class-consciousness` (`intensity`), `social-class/national-identity` (`intensity`),
`social-class/agitation` (domain `[0, ∞)`, same D-1-class hazard as Territory's
`rent_spike_multiplier`/Metabolism's `entropy_factor` — though `agitation`'s *value* is never
computed by this system, only read-passed-through, so this particular hazard binds whichever
system ports `ConsciousnessSystem`, not Solidarity itself). This flattening is **favorable**, not
merely a workaround: once the three fields are independent, `update-node` need touch only
`class-consciousness` — the entire read-the-whole-struct-to-change-one-field dance in
`_update_ideology_class_consciousness` (solidarity.py:45-75) **collapses to nothing**, and the
"default to `0.5`/`0.0` if the pre-existing dict lacks the key" oddity (§2 Step 4) disappears with
it, since each flat field would carry its own model default independently.

**Enum discriminant flag: none.** Unlike Territory, this system reads/writes no `deffield`-eligible
enum-typed attribute — `EdgeType.SOLIDARITY` is a query filter (an `EnumKind::EdgeType` operand,
already landed as a query-head argument per grammar.rs:199), never a stored payload value.

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `float`). Grep-confirmed zero `exp`/`log`/`sigmoid`/`pow`
calls anywhere in `solidarity.py`, `formulas/solidarity.py`, or `kernel/node_access.py` —
**this system has zero libm-nondeterminism hazard**, matching Territory and unlike Metabolism.
Shapes, in execution order:

1. **Threshold comparisons (×2, duplicated across caller and formula):** `solidarity_strength <= 0` (solidarity.py:135, formulas/solidarity.py:33) and `source_consciousness <= activation_threshold` (solidarity.py:143, formulas/solidarity.py:33) — plain `<=`. The formula's own copy of both guards is provably dead at the call site (the caller already filtered), but is transcribed port-as-is since it is the frozen behavior.
2. **Subtractive difference + multiply:** `solidarity_strength * (source_consciousness - target_consciousness)` (formulas/solidarity.py:36) — one subtract, one multiply, no bare literal involved (all three operands are runtime values).
3. **Magnitude comparison:** `abs(delta) < services.defines.NEGLIGIBLE_TRANSMISSION` (solidarity.py:160) — `abs()` + `<`. `abs` over a `Value::Real` is a standard BSL arithmetic op (no special hazard); the define itself is unbounded-above (`[0, ∞)`), same domain-shape note as item 5 below, but since it's compared (not multiplied against a Currency/Ratio-typed operand) this does **not** hit the D-1-class scale-op hazard Territory/Metabolism recorded — a plain Real comparison against a plain Real `:const` is unconditionally portable.
4. **Additive accumulation:** `target_consciousness + delta` (solidarity.py:164) — one add.
5. **Clamp, ONE consistent implementation (favorable contrast with Territory's two-clamp inconsistency):** `max(0.0, min(1.0, new_consciousness))` (solidarity.py:165) — the **only** clamp site in this system, so there is no cross-site inconsistency to transcribe (unlike Territory's Phase-1-vs-Phase-3 clamp mismatch). Two **bare non-integer literals**, `0.0` and `1.0` — the same "no bare non-integer literal" BSL parser constraint Territory flagged; needs `c`-suffixed consts (`0.0c`/`1.0c`) or the Real-zero-promotion idiom.
6. **Chained comparison (syntax, not arithmetic):** `old_consciousness < mass_awakening_threshold <= new_consciousness` (solidarity.py:190) — Python's implicit two-inequality chain. BSL's `<cmp>` grammar (per bsl-language.rst) is binary per comparison; this transcribes mechanically to `(and (< old thresh) (<= thresh new))`, a syntax-only port-as-is note, not a semantic hazard.
7. **Bare non-integer literals in the ideology-rebuild defaults:** `ideology.get("national_identity", 0.5)` and `ideology.get("agitation", 0.0)` (solidarity.py:66-67, 73-74) — two more bare literals (`0.5`, `0.0`), though this whole helper is subsumed by the favorable content-modeling flattening in §3 (once `ideology` is 3 flat fields, this default-fill logic has no BSL analogue to transcribe at all).
8. **Real→Int demotions: none.** Grep-confirmed zero `int(...)` casts anywhere in this system — a favorable contrast with Territory's two `floor`-class demotions.
9. **Currency-mixing multiplies: none.** No operand in this system is `Currency`-typed in either the Python or BSL sense — `solidarity_strength` is `Coefficient` `[0,1]`, `class_consciousness` is a plain probability-shaped float. The D-1-class scale-op hazard (Territory's `rent_spike_multiplier`, Metabolism's `entropy_factor`) simply does not arise anywhere in this system's own arithmetic.

**Net float-op assessment: this system's arithmetic is the cleanest of the three inventoried so far
— no libm, no Int demotion, no clamp inconsistency, no Currency-scale hazard.** Every hazard this
system does carry is either (a) the universal bare-literal transcription tax, or (b) entirely
upstream of the arithmetic, in the fact that its operands are edge-resident (§6).

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 8.0** (`solidarity.py:91`), confirmed against `_SYSTEM_CLASSES`
  (`simulation_engine.py:328-363`) and per-file `position: ClassVar[float]` greps: `Vitality(1.0) →
  Territory(2.0) → Substrate(2.5) → Production(3.0) → TickDynamics(4.0,
  domain/economics/tick/system.py) → ReserveArmy(5.0) → Community(6.0) → Lifecycle(7.0) →
  Solidarity(8.0) → ImperialRent(9.0) → Transport(9.5) → ...`. `tests/unit/engine/test_system_order.py:132-138`
  additionally pins the *relative* order as a behavioral contract: `test_solidarity_runs_before_extraction`
  asserts Solidarity's index precedes ImperialRent's.
- **Reads from a same-tick prior system:**
  - `active` (`SOCIAL_CLASS`) — written by `VitalitySystem` (@1.0) on death
    (`engine/systems/vitality.py:169`, `updates: dict[str, Any] = {"active": False}`). A genuine
    same-tick channel: a class that dies at position 1.0 is correctly excluded from transmission at
    position 8.0 the same tick.
  - `solidarity_strength` (edge) — `CommunitySystem` (@6.0) *can* amplify it via
    `_amplify_solidarity_edges` (`engine/systems/community.py:527-576`,
    `graph.update_edge(..., solidarity_strength=amplified, ...)` at line 574) before Solidarity
    reads it, three positions later the same tick. **But this channel is itself dormant on every
    canonical scenario** — `CommunitySystem`'s own coverage-gap row
    (`tools/regression_scenarios.py:2849-2855`) states its `step()` body "returns on its first or
    second guard clause every tick" because `services.community_hypergraph is None` and no
    `MEMBERSHIP` edges are seeded, so `_amplify_solidarity_edges` never actually runs in the
    canonical estate.
  - `ideology.class_consciousness` — **not** written same-tick by anything before position 8.0;
    the primary drift system, `ConsciousnessSystem`, runs at position 17.0 (`ideology.py:109`),
    strictly downstream. So the `class_consciousness` this system reads on both endpoints is always
    the *previous tick's* committed value, never a same-tick write.
- **Writes consumed later this tick / downstream ticks:**
  - `SOCIAL_CLASS.ideology` (→ `class_consciousness`) — read immediately next this same tick by
    `ImperialRentSystem` (@9.0, `engine/systems/economic.py:286`,
    `consciousness = class_consciousness_from_node(worker_attrs)`), and downstream (same tick or
    next tick, depending on each reader's own position relative to 8.0) by: `StruggleSystem`
    (@16.0, `struggle.py:404,414,561`), `ConsciousnessSystem` itself (@17.0, `ideology.py:81`, which
    further drifts the same value this system just wrote), `AllegianceSystem` (@17.42,
    `allegiance.py:290`), `ElectoralSystem` (@17.45, `electoral.py:404`),
    `WealthDistributionSystem` (@21.5, `wealth_distribution.py:178`), `EpistemicHorizonSystem`
    (@22.0, last position, `epistemic_horizon.py:96`). `class_consciousness_from_node` is a shared
    accessor used identically by three of these (`node_access.py:23` names `SolidaritySystem`,
    `StruggleSystem`, `ImperialRentSystem` as the three original copies it consolidated).
  - **`EventType.CONSCIOUSNESS_TRANSMISSION`/`MASS_AWAKENING`** — consumed only by the
    narrative/AI-observer layer (§2 Step 5's citation list); no `engine/systems/*.py` branches on
    either.
- **Downstream writers of `solidarity_strength` this system will read on a *later* tick** (not
  same-tick, since all run after position 8.0): `_mass_work.py` (OODA action resolution, @14.0,
  `engine/actions/_mass_work.py:56,109-121`, gain capped at `_MAX_SOLIDARITY_STRENGTH`);
  `doctrine.py` (@14.7, `engine/systems/doctrine.py:120-138`, per-tick multiplicative decay,
  `solidarity_strength=max(0.0, strength * (1.0 - decay_rate))`); `StruggleSystem` (@16.0,
  `struggle.py:384-398`, solidarity gain from shared struggle/uprising — the George Floyd Dynamic,
  `tests/integration/mechanics/test_george_floyd_dynamic.py`).
- **Context/service usage with no BSL equivalent:** `context.get("tick", 0)` (`solidarity.py:172`)
  — used only to stamp the emitted `Event`'s `tick` field, not to gate any computation; trivial,
  the tick number is already an ambient BSL rule-evaluation input.
- **Peripheral, out-of-path reader (not in this system's own execution, noted for completeness):**
  `engine/event_evaluator.py:147-176` (`_collect_edge_value`) reads `solidarity_strength` off
  `SOLIDARITY` edges for a **separate**, legacy scripted-event-template condition system
  (`models/entities/event_template.py`) — never called from `SolidaritySystem.step()` itself, out
  of scope for this port.
- **DORMANCY on canonical scenarios — two distinct, reconcilable findings:**
  1. **The declared coverage-gap doc is explicit and code-verified for the "five original"
     scenarios.** `tools/regression_scenarios.py:2832-2840` (the `SolidaritySystem` row of
     `COVERAGE_GAPS_DATA`): *"every SOLIDARITY edge in all five scenarios has
     solidarity_strength=0.0 (imperial_circuit's scenario-seed default; two_node has no SOLIDARITY
     edge at all); the transmission loop's 'if solidarity_strength <= 0: continue' skips every edge
     every tick, so CONSCIOUSNESS_TRANSMISSION/MASS_AWAKENING never fire."* "Five original" =
     `imperial_circuit`, `two_node`, `starvation`, `glut`, `fascist_bifurcation`
     (`regression_scenarios.py:156-192`'s comment: *"five original + single_county, Task 8/E2a"*).
     Verified at the source: `create_imperial_circuit_scenario`'s `SOLIDARITY` edge
     (`engine/scenarios/_legacy.py:447-456`) takes `solidarity_strength: float = 0.0` as a default
     parameter (declared at `_legacy.py:262`) and no `SCENARIOS` entry overrides it
     (`starvation`/`glut`/`fascist_bifurcation` only override `economy.*`/`consciousness.*` defines,
     never the scenario-seed parameter); `two_node` (`_legacy.py:46-120`) seeds no `SOLIDARITY`
     edge at all.
  2. **This is NOT true of every canonical scenario registered in the current `SCENARIOS` dict —
     a fact the stale doc's own scope note ("five original + single_county") already signals, but
     is worth stating explicitly since the row text alone reads as a blanket claim.** Two later
     Program-25 electoral-golden scenarios seed a live, nonzero `SOLIDARITY` edge via the
     `_solidarity()` helper (`engine/scenarios/electoral_goldens.py:156-163`):
     `_solidarity(_WORKER, "C005", 0.4)` in `create_debs_scenario`
     (`electoral_goldens.py:474`) and `_solidarity(_WAYNE_WORKER, "C006", 0.4)` in
     `create_bernie_valve_scenario` (`electoral_goldens.py:534`). Both scenarios' source endpoint
     (`_WORKER`/`_WAYNE_WORKER`) inherits `class_consciousness=0.5` from the underlying
     `two_node`/`single_county` substrate's default `worker_ideology=0.0` →
     `IdeologicalProfile.from_legacy_ideology(0.0)` mapping (`social_class.py:113-116`, "ideology=0
     (neutral) → class_consciousness=0.5"), which is **already above** the `0.3` activation
     threshold at tick 0 (`from_legacy_ideology`'s mapping table, `social_class.py:113-116`) — so both
     of `SolidaritySystem`'s hard gates (`solidarity_strength > 0`,
     `source_consciousness > activation_threshold`) are satisfied by construction at scenario seed
     time. **UNVERIFIED beyond this:** whether `CONSCIOUSNESS_TRANSMISSION` actually fires end-to-end
     across a live `debs`/`bernie_valve` run depends on the two endpoints' consciousness trajectories
     staying apart (delta ≥ `negligible_transmission`) as the run proceeds — not run here (read-only
     mandate) — but the necessary tick-0 preconditions are demonstrably present, unlike the five
     original scenarios where `solidarity_strength=0.0` forecloses transmission unconditionally. A
     port's conformance fixtures should treat `debs`/`bernie_valve` as a possible harvest source for
     a "solidarity engages" vector, pending that live check, rather than assuming (as the five-scenario
     doc alone would suggest) that no canonical scenario ever exercises this system.

## 6. BLOCKER ASSESSMENT

Adjudicated directly against the current dev tree (`rust/crates/babylon-bsl/src/evaluator.rs`,
`grammar.rs`, `declarations.rs`; `docs/reference/bsl-language.rst` §2.10), verified live in this
session (§1's citation list).

| Computation | Verdict | Detail |
|---|---|---|
| Step 1 — SOLIDARITY edge selection + liveness/infrastructure gate (`solidarity.py:121-136`) | **BLOCKED — dyadic edge lane (query slice 2)** | Obtaining *any* `SOLIDARITY`-typed `EdgeRef` at all requires the `edges` query head (`(edges EdgeType/SOLIDARITY)`, bsl-language.rst §2.6/§2.10's own worked `EdgeType/EXPLOITATION` example, line 1886), which `evaluator.rs:504` lists in `UNSERVED_EXPRESSION_HEADS` tagged `"slice 2"`. There is no `neighbors`-based reformulation: `neighbors` (landed, `SERVED_QUERY_HEADS`, evaluator.rs:527) yields `NodeRef`s only (bsl-language.rst:1273's result-type table — `EdgeRef` is `edges`'/`edge-between`'s result type exclusively), so it cannot surface the edge itself. `solidarity_strength` is irreducibly edge-resident data (§3) — this step cannot begin without slice 2, full stop. The `active` liveness reads (`field-of` over the endpoints' `NodeRef`s) would be trivially servable today (Slice 1 landed) **if** the endpoints were already in hand, but they are only reachable via the blocked edge. |
| Step 2 — source activation gate (`solidarity.py:138-144`) | **BLOCKED — transitively, on Step 1** | The comparison itself (`field-of` a `NodeRef` against a `:const activation-threshold`) is mechanically identical to Territory's already-portable `heat >= eviction_threshold` shape and would be `PORTABLE NOW` in isolation — but the source `NodeRef` is only reachable by first resolving the `SOLIDARITY` `EdgeRef` (Step 1), so this step inherits the block. |
| Step 3 — transmission delta + negligible-delta gate (`solidarity.py:146-161`, `formulas/solidarity.py:36`) | **BLOCKED — dyadic edge lane (query slice 2), directly** | `solidarity_strength * (source_consciousness − target_consciousness)` needs `(field-of it solidarity/strength)` where `it` is the `EdgeRef` from Step 1 — `field-of` over an `EdgeRef` referent is explicitly named unreachable today: *"an `EdgeRef` referent is unreachable today (no expression form produces one yet ... so `field-of` over one is unexercised until slice 2's `edges`/`edge-between` land)"* (evaluator.rs:1185-1191). This is the system's most direct hit on the named gap. The pure arithmetic (`abs`, `-`, `*`, `<`) carries **no additional math-side blocker** — once the two operand reads land, the formula itself is immediately expressible with a trivial `defconst` for `activation_threshold`/`negligible_transmission`, no D-record needed for the math. |
| Step 4 — clamp + node write (`solidarity.py:163-169`) | **BLOCKED — transitively, on Step 1; PORTABLE WITH D-RECORD once unblocked** | `update-node` against a computed `NodeRef` (here, `edge.target_id`) is the exact landed shape from ADR197/Territory's blocker-table resolution ("update-node accepts a computed NodeRef… against the population transfer"), so mechanically this write is already proven servable through the real `run_once_into` seam. It is blocked here only because the target `NodeRef` is reached via the blocked `EdgeRef` (`edge.target_id`), not because `update-node` itself is missing. Once Step 1 lands, this step needs exactly one D-record: the `ideology`-nested-dict → three-flat-`deffield`s content-modeling decision (§3), which is *favorable* (it deletes the read-modify-write dance entirely rather than adding complexity) but still a transcription decision with no existing precedent in a landed pack. The clamp (`max(0.0, min(1.0, ...))`) transcribes as a nested `if`, matching every landed pack's convention (no scalar `min`/`max` in the grammar) — same idiom Territory already established, no new D-record needed for the clamp shape itself. |
| Step 5 — event emission (`solidarity.py:171-202`) | **PORTABLE WITH D-RECORD (WS1 ledger, not a blocker)** | `emit` exists in effect position and every landed pack uses it — mechanically servable. But per the CURRENT BSL surface, `TickReport` carries no event log, so both `CONSCIOUSNESS_TRANSMISSION` and `MASS_AWAKENING` are WS1 (#502) ledger rows: expressible today, unpinnable by any conformance golden until the event-log carrier lands. Not a blocker on the computation itself, but the pack cannot claim byte-parity on its event stream yet. |
| `ideology` nested-dict → flat-field content model (cross-cutting, §3) | **D-RECORD, favorable** | Not itself a blocker (it is a content-modeling choice, not a missing language feature) but is a genuinely new decision this system introduces that Territory's all-flat-attribute case never needed: the eventual pack must declare `social-class/class-consciousness`, `social-class/national-identity`, `social-class/agitation` as three independent `deffield`s. `agitation`'s `[0, ∞)` unbounded domain inherits the same D-1-class scale-op caution Territory/Metabolism recorded, but binds only whichever system computes `agitation` (`ConsciousnessSystem`), not this one, since Solidarity only reads-and-passes-through the other two fields. |
| Multi-inbound-edge write-visibility semantics (cross-cutting, §5's caveat, `test_law_solidarity.py:49-60`) | **OPEN QUESTION, not adjudicable today** | The frozen Python processes `SOLIDARITY` edges **sequentially** via `for edge in graph.query_edges(...)`, each iteration's `update_node` write immediately visible to the next iteration when multiple edges share a target (order-dependent — the test suite explicitly declines to assert an edge-count-invariant closed form for this case, per its own docstring caveat). Whether a future `for-each` over `(edges EdgeType/SOLIDARITY)` (once slice 2 lands) preserves this same sequential in-place visibility, or instead batch-reads pre-tick state before applying any writes (Territory's Phase-3 collect-then-apply pattern), is a genuine open design question for the slice-2 train itself — this inventory cannot verify it since the head is unserved on the current tree. Flagged for the slice-2 design gate, not resolvable by content-side D-record. |

**No RESERVED-LINE surface in this system.** `SolidaritySystem` contains no doctrine-tree content, no
National Question parameters, and no outcome-definition logic — it is pure mechanical
consciousness-diffusion math over a coefficient the player/AI organizes to build (`solidarity_strength`
itself is written elsewhere, by OODA mass-work actions and Struggle's uprising gain, both
out of this system's scope). Confirmed by full line-by-line read of `solidarity.py`.

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_solidarity_system.py` | 541 | **Primary conformance-oracle candidate.** TDD-red-phase-originated exhaustive unit coverage per the module docstring: basic transmission, unidirectional flow, `solidarity_strength=0` (Fascist Bifurcation) skip, dead-node skip, activation-threshold gating, event emission (`CONSCIOUSNESS_TRANSMISSION`/`MASS_AWAKENING`), multiple-inbound-edges ordering (`TestSolidaritySystemEdgeCases::test_multiple_solidarity_edges`, cited by name from the laws file). |
| `tests/unit/engine/laws/test_law_solidarity.py` | 281 | **Behavioral-contract laws — the highest-value conformance-oracle candidate.** Hypothesis property-based tests pinning L1 (clamp: post-step `class_consciousness` always in `[0,1]` for any inputs), L2 (directional convergence, never overshoot — the delta is a convex combination of target/source, algebraic proof in the module docstring), L3 (inactivity below activation threshold — byte-for-byte unchanged, no event), L4 (inactivity on dead nodes / zero infrastructure). Explicitly states what it does *not* assert (no edge-count-invariant "total consciousness conserved" law — this is diffusion, not transfer; source is never mutated). Each law traces to an exact file:line range in its own docstring — ideal transcription-order reading for a port author. |
| `tests/unit/formulas/test_solidarity.py` | 231 | Unit coverage of `calculate_solidarity_transmission` in isolation — a secondary conformance-oracle candidate for the pure-math half of Step 3. |
| `tests/integration/mechanics/test_proletarian_internationalism.py` | 396 | Full-engine (`Simulation`) multi-tick integration: revolutionary-scenario cumulative awakening, Fascist Bifurcation (`sigma=0` → no transmission), event emission through the real engine loop, turn-order integration. `pytest.mark.theory_solidarity`. A candidate end-to-end conformance vector, not unit-scoped. |
| `tests/integration/mechanics/test_george_floyd_dynamic.py` | 528 | **Tangential, not a direct SolidaritySystem oracle.** Tests `StruggleSystem`'s solidarity-*infrastructure-building* side (agitation → spark → uprising → `solidarity_strength` gain) — the upstream write this system later reads on a subsequent tick. Useful for understanding the write side of the cross-system channel (§5), not for SolidaritySystem's own transmission math. |
| `tests/unit/kernel/test_node_access.py` | 70 | Unit coverage of the shared `class_consciousness_from_node` accessor — schema/accessor-level, small conformance-oracle candidate for Step 2/3's read shape. |
| `tests/unit/engine/test_system_order.py` | 300 | Pins tick position 8 and the relative-order contract (`test_solidarity_runs_before_extraction`) — an ordering conformance oracle for §5, not a computation oracle. |
| `tests/unit/engine/test_event_conversion.py` | 1774 | Schema-level only: `CONSCIOUSNESS_TRANSMISSION`/`MASS_AWAKENING` → typed-Pydantic-event conversion round-trip (`_convert_bus_event_to_pydantic`). Exercises the full 100-value `EventType` enum's conversion table, not solidarity-specific behavior — not a conformance oracle for this system's computation. |
| `tests/unit/dialectics/test_connectivity_instance.py` | 259 | Tests the `atomization_index`/`AdjointCylinder.balance` theory module `solidarity.py`'s docstring cross-references but never calls (§1). Out of scope for this port — no behavior-path overlap. |

**qa:regression byte-gate coverage.** Per §5's dormancy finding, `tools/regression_test.py`'s
graph-content-hash gate would catch any unintended drift in this system's outputs on any canonical
scenario that exercises it — but on the five *original* scenarios (`imperial_circuit`, `two_node`,
`starvation`, `glut`, `fascist_bifurcation`), the system never engages at all
(`solidarity_strength=0.0` everywhere), so that coverage is currently vacuous for this system's
actual transmission logic on that subset. `debs`/`bernie_valve` (Program-25 electoral goldens) seed
the necessary preconditions (§5) but whether the byte-gate's dense/checkpoint goldens actually
observe a nonzero `CONSCIOUSNESS_TRANSMISSION`-driven `class_consciousness` delta across their runs
is unverified here. A port's Phase-2+ conformance fixtures should plan on hand-built `.bscn`
fixtures (matching `test_law_solidarity.py`'s own fixture-construction style,
`_worker`/`_solidarity_edge` helpers at lines 85-113) rather than assuming canonical-scenario
harvest, pending that live check.

---

## Adjudication (2026-08-12)

Adjudicated against the dev tree at `9324482f` (two merges ahead of this inventory's
authoring HEAD: #552, #553). Three corrections, three confirmations.

1. **CONFIRMATION — the slice-2 spine holds exactly as written, verified at the byte.**
   `UNSERVED_EXPRESSION_HEADS` (`rust/crates/babylon-bsl/src/evaluator.rs:503-512`) is
   `[("edges","slice 2"), ("edge-between","slice 2"), ("the","slice 2"), ("hyperedges","slice 3"),
   ("members-of","slice 3"), ("hyperedges-of","slice 3"), ("metric-of","slice 3"),
   ("membership-field-of","slice 4")]`; `SERVED_QUERY_HEADS` (`evaluator.rs:527`) is exactly
   `["nodes", "neighbors"]`. `eval_field_of` (`evaluator.rs:1210-1222`) matches only
   `Value::NodeRef` and `Value::HyperedgeRef`, refusing everything else with *"edge referents
   ride slice 2"*. There is no route to an `EdgeRef` on current dev. The verdict's spine is
   correct.

2. **CONFIRMATION — and a sharpening on WHY the datum is irreducibly edge-resident.**
   `GraphSubstrate::add_edge(&mut self, edge_type, from, to, strength: f64)`
   (`rust/crates/babylon-graph/src/substrate.rs:111-117`) gives a dyadic edge exactly ONE f64,
   surfaced to content as the implicit `<edge-type>/strength` field (D32,
   `declarations.rs:13, 317-320`). So the natural port models `solidarity_strength` **as** the
   edge's own `strength` — which does not soften the blocker, it localises it: the read path is
   still `field-of <EdgeRef> solidarity/strength`, still slice 2. Worth recording because it
   means Solidarity needs **no** edge-attribute-storage widening (D35/D65) the way
   `ImperialRentSystem`'s `value_flow` does — slice 2 alone unblocks this system, which is a
   materially cheaper unblock than that sibling inventory's.

3. **CORRECTION — §5's "the necessary tick-0 preconditions are demonstrably present" on
   `debs`/`bernie_valve` is wrong; transmission provably does NOT fire at tick 0.** In BOTH
   scenarios the SOLIDARITY target is a construction-time CLONE of the source:
   `_with_worker_twin` is `state.entities[source_id].model_copy(update={"id": twin_id})`
   (`electoral_goldens.py:128`), and `debs` builds `C005` that way at `:426` before giving
   `_WORKER` and `C005` **identical** `_voter(...)` parameters (`:441-457` — same
   `{_SOCDEM: 0.45, _LIBERAL: 0.25}`, `population=2`, `agitation=0.5`, `repression=0.5`,
   `wealth=0.35`); `bernie_valve` builds `C006` at `:503` and gives all three workers the same
   `_voter(...)` params bar wealth (`:510-518`). So at seed
   `source_consciousness − target_consciousness == 0.0` exactly, `delta == 0.0`, and
   `abs(delta) < NEGLIGIBLE_TRANSMISSION` (0.01, defines.yaml:186) takes the `continue` at
   `solidarity.py:160-161`. Two of the three gates are satisfied by construction; the **third is
   provably closed at tick 0**. Transmission can engage only once the twins' `class_consciousness`
   diverges by ≥ 0.025 (`0.4 × gap ≥ 0.01`) — which is exactly what those scenarios are built to
   produce, but it is a run-time fact, not a seed-time one.
   **The other half of §5.2 stands and is strengthened:** `tests/baselines/debs.json` and
   `tests/baselines/bernie_valve.json` both exist, so both scenarios ARE inside the
   `qa:regression` byte-gate, and `graph_content_hash` hashes every node attribute of the
   projection including `ideology` (`tools/regression_test.py:961-964`). The gate's *reach* over
   this system is established; only its *engagement* is unverified. A port author should plan the
   conformance fixture on that basis: hand-built `.bscn`, with `debs`/`bernie_valve` as a
   post-divergence harvest candidate, never a tick-0 one.

4. **CORRECTION — §6's last row ("Multi-inbound-edge write-visibility semantics — OPEN QUESTION,
   not adjudicable today") IS adjudicable on current dev, and the answer goes against the frozen
   shape.** `tick.rs`'s module doc records the ruling as landed: *"**Superseded (Task 12, P27
   Phase 2 query-evaluation plan, 2026-08-11):** this section used to say each subject reads
   through the same graph it mutates and sees prior subjects' mutations, matching the frozen
   Python engine's in-place behaviour. That was an admitted implementation/spec divergence (D-row
   Q1), not a ruling — §4.2 chapter C4 says 'all firings of one rule observe the same pre-state
   … and the effects they collect are applied in that subject order.' `run_tick` now follows the
   spec: it runs in two passes, collecting every subject's writes … against the SAME pre-tick
   graph before applying any of them"* (`rust/crates/babylon-bsl/src/tick.rs:41-52`). So the
   semantics is decided, not open: a slice-2 `for-each` over `(edges EdgeType/SOLIDARITY)` will
   **not** reproduce the frozen loop's sequential in-place visibility, and a target with two
   inbound SOLIDARITY edges will take a last-write-wins `set` rather than the frozen loop's
   cumulative application. That is a **named divergence the pack must D-record**, and it is
   pinned on the frozen side by `TestSolidaritySystemEdgeCases::test_multiple_solidarity_edges`
   (§7). Re-file this row from "flag for the slice-2 design gate" to "port-time D-record,
   semantics already ruled."

5. **CORRECTION — the `active` liveness read is NOT "trivially servable today"; `field-of` is
   type-blind, and no boolean is readable as a `Value::Bool` at all.** `field_of_node` returns
   `Ok(Value::Real(value))` unconditionally (`evaluator.rs:1281-1291`), and so does a `:field`
   binding for every non-enum declared type — `bind_field_value` returns `Value::Real(stored)`
   except in the `BslType::Enum` branch (`tick.rs:312-327`). A `bool`-declared field therefore
   evaluates to `Value::Real(0.0|1.0)`, which `as_bool` refuses where a `<cond>` is required
   (`evaluator.rs:1315-1320`) and which `apply_equality` refuses against a `#t`/`#f` literal —
   *"equality is defined within one lane only"* (`evaluator.rs:1620-1628`). The only expressible
   form is a numeric comparison, `(= (field-of it social-class/active) 1)`, over a 0/1-encoded
   `int` field. Landed content already does exactly this and documents it in-file:
   `vitality-conformance.bscn:20` and `vitality-lifecycle-combined-conformance.bscn:34` —
   *"0/1 rather than #t/#f"*. This binds Step 1's `active` gate and Step 4's write shape; add it
   to §3's type inventory and to §6's Step-1 row (it does not change the row's verdict, which is
   BLOCKED for a prior reason).

6. **CONFIRMATION — everything else in §2/§4/§5 re-verified and standing.** Tick position 8.0
   (`solidarity.py:91`), between `LifecycleSystem` (7.0) and `ImperialRentSystem` (9.0), against
   `_SYSTEM_CLASSES` (`simulation_engine.py:328-360`) — which `_DEFAULT_SYSTEMS` derives by
   sorting on `position` (`simulation_engine.py:376-378`), so the tuple order and the tick order
   agree. `solidarity_strength` read off the edge at `solidarity.py:133`; the single clamp at
   `:165`; the chained comparison at `:190`; both `EventType` emissions at `:174` and `:192`.
   Cross-system spot-checks: the `ideology` write (`:169`) is consumed one position later by
   `ImperialRentSystem`'s `class_consciousness_from_node(worker_attrs)` at `economic.py:286`; the
   `active` read is written same-tick by `VitalitySystem` @1.0 at `vitality.py:169`
   (`updates: dict[str, Any] = {"active": False}`). The coverage-gap row is verbatim at
   `tools/regression_scenarios.py:2833-2841`. **No RESERVED-LINE surface** — independently
   confirmed: no doctrine content, no National Question parameter, no outcome definition in this
   system's reachable code. (Nit, not a correction: the header says 203 lines; `wc -l` reports
   202 — every in-body citation is correct.)

**FINAL VERDICT: BLOCKED — entirely, on BSL query-evaluation Slice 2 (the dyadic edge lane:
`edges`/`edge-between`/`field-of`-over-`EdgeRef`) — UPHELD.** Slice 2 alone unblocks this system:
unlike `ImperialRentSystem`, Solidarity needs no D35/D65 edge-attribute-*storage* widening,
because its one edge datum fits the implicit `<edge-type>/strength` field. Two port-time
D-records are now named that the inventory did not carry: (i) boolean-as-0/1 `int` encoding for
every `active` read and write (correction 5), and (ii) the collect-then-apply divergence for
multi-inbound-edge targets, whose semantics is already ruled and landed, not open (correction 4).
The reader's own `ideology`-flattening D-record stands as filed and remains favorable.

**INADEQUATE-COVERAGE NOTE (narrow).** §1's Rust-side reading list covers `evaluator.rs`,
`grammar.rs` and `declarations.rs` but not `tick.rs` or `structural_verbs.rs` — which is why the
Task-12 two-pass ruling (correction 4) reads as an open question and the boolean lane
(correction 5) is unnamed. A re-read must add `tick.rs:41-52` (subject/pre-state semantics),
`tick.rs:312-327` (`bind_field_value`'s type rendering) and `structural_verbs.rs:1196-1234`
(`numeric_write_value`, the one funnel every node write crosses) before any row is graded on the
read/write value lane.
