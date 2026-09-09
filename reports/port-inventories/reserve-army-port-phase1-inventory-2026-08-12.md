# ReserveArmySystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `ReserveArmySystem` (#5, `reserve_army.py`, 148 lines) is small but its
one substantive computation — `DefaultWagePressureCalculator.compute_wage_pressure`, a
hand-rolled bounded/normalized logistic sigmoid over `math.exp` — is not merely undeclared in
BSL, it is **the named example** of a prohibited construct: `bsl-language.rst` §3.10 and
**ADR188 Row 7** (Director-ratified 2026-08-10) identify "the wage-pressure sigmoid" by name as
one of three `exp` call sites that **must re-derive as a measure**, and
`declarations.rs:116` mechanically refuses an intrinsic literally named `sigmoid`
(`E-LOAD-024`). This is not an open question awaiting escalation — it is a closed Director
ruling with undone port-time design work. Everything else in the system (territory iteration,
the multiplicative wage write, event emission) is mechanically portable once that redesign
lands, except the `border_regime` valve, which is blocked separately on the missing
edge-attribute lane (BSL query Slice 2) and has no BSL representation for the graph-level
`policy_overlays` side-channel register at all. The system is dormant on every canonical
`qa:regression` scenario today (declared gap, `tools/regression_scenarios.py:2802-2808`).

**Verdict: BLOCKED — on ADR188 Row 7's undesigned wage-pressure→measure re-derivation** (the
system's one real computation is a RULED-prohibited stipulated sigmoid, not a missing-grammar
gap); the border-valve throttle is separately BLOCKED on the edge-attribute query lane (Slice 2)
and an unrepresented graph-side-channel register; the territory-iteration shell, wage write, and
event emission are PORTABLE NOW (mechanically) once a wage-pressure value exists to write.

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/reserve_army.py` | 148 | **The target.** `ReserveArmySystem`, one `step()` + one static helper `_border_valve`. Imports `DefaultWagePressureCalculator` directly (line 12) and calls it (line 94) — the only `domain/` module `step()` invokes. |
| `src/babylon/domain/economics/reserve_army/calculator.py` | 65 | `DefaultWagePressureCalculator.compute_wage_pressure` — the bounded-sigmoid wage-pressure formula. Instantiated and called by `ReserveArmySystem.step()` directly (reserve_army.py:60,94). **The system's one substantive computation.** |
| `src/babylon/engine/systems/policy.py` | (relevant: 86-159, 368-404) | `POLICY_OVERLAYS_ATTR` constant + the `{sovereign_id: {axis: {"magnitude", ...}}}` register shape `_border_valve` reads (policy.py:91-95, 399-404). `PolicySystem` (@17.47) is the sole writer — a different System, not invoked by `reserve_army.py`. |
| `src/babylon/kernel/graph_protocol.py` | (relevant: 77-98, 258-273, 415-432) | `GraphProtocol.get_node`/`update_node`/`query_nodes`/`query_territory_claims` signatures — every graph-facing call `reserve_army.py` and `_border_valve` make. |
| `src/babylon/topology/graph.py` | (relevant: 942-959) | Concrete `BabylonGraph.query_territory_claims` — CLAIMS-edge scan, sorted `(-control_level, source_id)`; the data `_border_valve` reads. |
| `src/babylon/kernel/system_base.py` | 99-117 | `SystemBase._wrap_graph` — a type-check-and-return-unchanged guard (no adapter wrapping); `protocol` in `reserve_army.py:57` is the same `BabylonGraph` object. |
| `src/babylon/config/defines/economy_labor.py` | 45-151 | `ReserveArmyDefines` Pydantic model — 6 coefficient fields; only 3 (`sigmoid_k`, `sigmoid_r0`, `wage_pressure_ceiling`) are read by `DefaultWagePressureCalculator`; the other 3 belong to a **different** calculator (`DefaultAccumulationLoopCalculator`, see §5). |
| `src/babylon/data/defines.yaml` | 413-419 | Player-editable `reserve_army:` block — all 6 field values/domains. |
| `src/babylon/models/entities/territory.py` | 211-239 | `Territory` Pydantic entity — `median_wage` (`Currency`), `reserve_ratio` (`float [0,1]`), `reserve_army_stock` (`float ≥0`) field declarations. `wage_pressure` is **not** a declared field (see §3). |
| `src/babylon/models/enums/topology.py` | 61 | `NodeType.TERRITORY = "territory"` — the query filter `reserve_army.py:71` uses. |
| `src/babylon/models/enums/politics.py` | 16-49 | `PolicyAxis.BORDER_REGIME = "border_regime"` — the axis key `_border_valve` looks up (reserve_army.py:141). |
| `src/babylon/models/enums/events.py` | 109 | `EventType.RESERVE_ARMY_PRESSURE = "reserve_army_pressure"` — the one event this system emits. |
| `src/babylon/models/events/dispossession_payloads.py` | 69-80 | `ReserveArmyPressureEvent` — the typed payload model (`territory`, `reserve_ratio`, `wage_pressure`, `median_wage`). |
| `src/babylon/kernel/event_bus.py` | 33-50, 134-148 | `Event` frozen dataclass (`type: str`, `payload: dict[str, Any]`) and `EventBus.publish`. |
| `src/babylon/models/world_state.py` | 94-134 | `TERRITORY_EXCLUDED_FIELDS` — `wage_pressure` (line 118) is dropped on every `WorldState.from_graph()` reconstruction; it never survives a round-trip. |
| `src/babylon/engine/simulation_engine.py` | 328-364 | `_SYSTEM_CLASSES` tuple — confirms tick position 5.0, directly after `TickDynamicsSystem` (4.0). |

**Not invoked by `reserve_army.py`'s `step()`, but load-bearing for §5:**
- `src/babylon/domain/economics/reserve_army/accumulation.py` (176 lines, `DefaultAccumulationLoopCalculator`) — a *different* calculator in the same `domain/economics/reserve_army/` package, invoked by `TickDynamicsSystem._compute_accumulation_loop` (`tick/system/__init__.py:1243-1334`), NOT by `ReserveArmySystem`. It is the producer of the `reserve_ratio` this system reads (§5).
- `src/babylon/domain/economics/reserve_army/data_sources.py` (35 lines) / `types.py` (76 lines) — the `ReserveArmyState`/`ReserveArmyDynamics` Pydantic models and the SQLite-backed `ReserveArmyDataSource` protocol, consumed by a **second, independent** wage-pressure application inside `TickDynamicsSystem._compute_vol1_layer` (`tick/system/__init__.py:1175-1241`), also not invoked by this system (§5).

**Reference BSL/normative sources read for this inventory:**
- `rust/crates/babylon-bsl/src/declarations.rs` lines 93-116, 700-731 — `DECLARABLE_INTRINSICS = ["exp", "log", "floor"]` and `PROHIBITED_INTRINSIC_NAMES = ["sigmoid"]`, with the comment naming ADR172 ruling 5 as the reason.
- `docs/reference/bsl-language.rst` lines 3195-3380 — §3.10's full "cap-legality is not doctrine-legality" argument, naming "a wage-pressure sigmoid" explicitly as one of three prohibited `exp` uses, and the twelve-row rider table (Row 7 = `exp`, Row 9 = `sigmoid`).
- `ai/decisions/ADR188_intrinsic_rider_slate_dispositions.yaml` — the Director's row-by-row ruling; Row 7 names "the wage-pressure sigmoid" and orders it to "RE-DERIVE AS MEASURES at the port."
- `ai/decisions/ADR172_amendment_ae_refoundation_ratified.yaml`, `ADR173_audit_and_stops_dispositions.yaml` — located but not needed beyond the citations already in `docs/reference/bsl-language.rst` and `ADR188`, which quote their operative rulings directly.

## 2. COMPUTATION CATALOG (execution order, `reserve_army.py:44-121`)

### Step 1 — Territory iteration + reserve_ratio gate (`reserve_army.py:71-81`)
- **(a)** Iterate every `territory` node; skip any whose `reserve_ratio` is absent, non-numeric, or `≤ 0`.
- **(b)** `reserve_ratio = float(data.get("reserve_ratio", 0.0))` (line 75, 79); `if reserve_ratio <= 0.0: continue` (line 80-81).
- **(c) Reads:** `TERRITORY.reserve_ratio` (default `0.0` if absent).
- **(d) Writes:** none.
- **(e) Defines:** none.
- **(f) Events:** none.

### Step 2 — Border-regime valve throttle (`reserve_army.py:83-91`, `_border_valve` at 123-147)
- **(a)** If a `policy_overlays` register exists, find the territory's top CLAIMS-holder (by `control_level` descending), read that sovereign's `border_regime` overlay magnitude, and shrink `reserve_ratio` by `(1 - magnitude)` — "a tighter border throttles the reserve army's replenishment" (comment, lines 86-88).
- **(b)** `overlays = protocol.get_graph_attr("policy_overlays", None)` (line 67). Per territory: `rows = graph.query_territory_claims(territory_id)` (line 135) → `sovereign_axes = overlays.get(rows[0][0])` (line 138) → `border = sovereign_axes.get("border_regime")` (line 141) → `magnitude = border.get("magnitude")` (line 144) → `min(1.0, max(0.0, float(magnitude)))` (line 147, a defensive re-clamp — `magnitude` is already `[0,1]`-constrained upstream by `PolicyAgendaItem.magnitude`, `policy.py:98`). Then `reserve_ratio *= (1.0 - border)` (line 89); re-checks `reserve_ratio <= 0.0` and continues if so (lines 90-91).
- **(c) Reads:** graph-level attr `policy_overlays` (a plain untyped nested dict, `{sovereign_id: {axis: {"magnitude", "enacted_tick", "promised", "delivered"}}}`, `policy.py:91-95`); `query_territory_claims(territory_id)` → CLAIMS edges' `control_level`/`legal_status` (edge attributes) + `source_id`.
- **(d) Writes:** none (mutates the local `reserve_ratio` variable only).
- **(e) Defines:** none (the `[0,1]` bound on `magnitude` is a Pydantic field constraint upstream, `PolicyAgendaItem.magnitude`, `policy.py:98`, not a `GameDefines` coefficient).
- **(f) Events:** none.
- **Pipeline-position note (verified against the comment, `reserve_army.py:62-66`):** `PolicySystem` writes `policy_overlays` at position 17.47; `ReserveArmySystem` reads it at 5.0. Since `5.0 < 17.47`, a same-tick read sees **last tick's** write (the I-ORD grain) — the comment states this explicitly and it matches every other same-tick-stale-read pattern documented elsewhere in the engine (e.g. `POLICY_OVERLAYS_ATTR`'s own docstring, policy.py:92-94).

### Step 3 — Wage-pressure sigmoid (`calculator.py:32-65`, invoked `reserve_army.py:94`)
- **(a)** Map `reserve_ratio ∈ [0,1]` to a wage-pressure coefficient in `[0, ceiling]` via a bounded, zero-shifted logistic sigmoid: higher reserve ratio ⟹ stronger downward wage pressure.
- **(b)** Exact formula (`calculator.py:41-65`):
  ```
  if reserve_ratio <= 0.0: return 0.0                                    # line 41
  exponent = -k * (reserve_ratio - r0)                                   # line 49
  exponent = max(min(exponent, 500.0), -500.0)                           # line 51 (clamp #1)
  raw = 1.0 / (1.0 + exp(exponent))                                      # line 52 — libm exp #1
  baseline_exponent = -k * (0.0 - r0)                                    # line 55 (== -k*(-r0), redundant subtraction, transcribed as-is)
  baseline_exponent = max(min(baseline_exponent, 500.0), -500.0)         # line 56 (clamp #2)
  baseline = 1.0 / (1.0 + exp(baseline_exponent))                        # line 57 — libm exp #2
  max_raw = 1.0 - baseline                                               # line 60
  if max_raw <= 0.0: return 0.0                                          # line 61-62
  normalized = (raw - baseline) / max_raw                                # line 64
  return ceiling * max(0.0, min(normalized, 1.0))                        # line 65 (clamp #3)
  ```
- **(c) Reads:** `reserve_ratio` (the Step-1/2 local, passed by value); `defines.sigmoid_k`, `defines.sigmoid_r0`, `defines.wage_pressure_ceiling`.
- **(d) Writes:** none (pure function; returns `float`).
- **(e) Defines:** `reserve_army.sigmoid_k` (20.0, `(0, 100]`), `reserve_army.sigmoid_r0` (0.08, `(0, 1]`), `reserve_army.wage_pressure_ceiling` (0.5, `(0, 1]`) — `economy_labor.py:62-96`, `defines.yaml:414-416`.
- **(f) Events:** none.
- **This is the system's one substantive computation, and it is the named subject of a Director ruling — see §6.**

### Step 4 — Wage multiply + node write (`reserve_army.py:98-107`)
- **(a)** If `wage_pressure > 0`, reduce `median_wage` multiplicatively and persist both `wage_pressure` and the (possibly throttled) `reserve_ratio` onto the node.
- **(b)** `if wage_pressure <= 0.0: continue` (line 95-96). `updates = {"wage_pressure": wage_pressure, "reserve_ratio": reserve_ratio}` (lines 100-103). `current_wage = data.get("median_wage", 0.0)`; `if isinstance(current_wage, (int, float)) and current_wage > 0.0: updates["median_wage"] = float(current_wage) * (1.0 - wage_pressure)` (lines 104-106). `protocol.update_node(node.id, **updates)` (line 107).
- **(c) Reads:** `TERRITORY.median_wage`.
- **(d) Writes:** `TERRITORY.wage_pressure` (graph-only, not a `Territory` model field — see §3), `TERRITORY.reserve_ratio` (re-stamped with the post-throttle value even when Step 2 never fired, since it's unconditionally in `updates`), `TERRITORY.median_wage` (only when `median_wage > 0.0` — a territory with `median_wage == 0.0` gets `wage_pressure`/`reserve_ratio` written but **not** `median_wage`).
- **(e) Defines:** none directly (consumes Step 3's output).
- **(f) Events:** none.

### Step 5 — Event emission (`reserve_army.py:109-121`)
- **(a)** Publish one `RESERVE_ARMY_PRESSURE` event per territory that received wage pressure, mirroring the post-update node state.
- **(b)** `services.event_bus.publish(Event(type=EventType.RESERVE_ARMY_PRESSURE, tick=tick, payload={"territory": node.id, "reserve_ratio": reserve_ratio, "wage_pressure": wage_pressure, "median_wage": updates.get("median_wage", data.get("median_wage", 0.0))}))` (lines 110-121). The `median_wage` payload field falls back to the **pre**-update `data.get("median_wage", 0.0)` only when `updates` has no `"median_wage"` key (i.e. the `current_wage > 0.0` guard in Step 4 failed) — so for a territory with `median_wage == 0.0`, the event still fires (since `wage_pressure > 0.0` was already established) and reports `median_wage: 0.0`, an honest mirror of the unwritten field, not a stale value.
- **(c) Reads:** the local `reserve_ratio`/`wage_pressure`/`updates`/`data` computed above.
- **(d) Writes:** none (event bus only).
- **(e) Defines:** none.
- **(f) Events:** `EventType.RESERVE_ARMY_PRESSURE` (`events.py:109`), payload-typed as `ReserveArmyPressureEvent` (`dispossession_payloads.py:69-80`).

**Events emitted by this system: exactly one distinct `EventType` — `RESERVE_ARMY_PRESSURE`.** Grep-confirmed single reference in `reserve_army.py` (line 112).

## 3. TYPE INVENTORY

Runtime storage note (same fact Territory's inventory established, re-verified for this file):
`BabylonGraph.update_node` is a plain dict merge (`topology/graph.py`, cf. Territory's own
citation at `:660-670`) — no type coercion or quantization at tick time. `Territory.median_wage`'s
`Currency` `SnapToGrid` quantization (`models/types.py:104-111`) applies only when the `Territory`
model is *instantiated* (seed / `WorldState.from_graph()`), never mid-tick, so every value this
system reads/writes in-graph is raw Python `float`.

| Attribute | Node/scope | Python model type | Domain | Category |
|---|---|---|---|---|
| `reserve_ratio` | TERRITORY | `float` (declared field, `territory.py:220-225`) | `[0.0, 1.0]` | unit-interval |
| `median_wage` | TERRITORY | `Currency` = `Annotated[float, ge=0.0, SnapToGrid]` (`territory.py:216-219`, `types.py:104-111`) | `[0.0, ∞)` | **unbounded real, money-semantic** |
| `wage_pressure` | TERRITORY | **not a declared `Territory` field** — graph-only, dropped by `TERRITORY_EXCLUDED_FIELDS` on every `from_graph()` (`world_state.py:118`); registered instead in the seam-liveness sentinel as `DECLARED_CONDITIONAL`, `dtype="float"` (`sentinels/seam/registry.py:385-397, 1216-1226`) | `[0.0, ceiling]` (functionally, `ceiling ≤ 1.0` by `wage_pressure_ceiling`'s own domain) | **ephemeral computed float — never round-trips past the tick boundary** |
| `reserve_army_stock` | TERRITORY | `float` (`territory.py:226-239`) | `[0.0, ∞)` | unbounded real (headcount-semantic, not read by this system — written by the cross-system accumulation loop, §5) |
| `policy_overlays` (graph-level attr) | graph | untyped `dict[str, Any]` (no Pydantic model — a plain nested dict `{sovereign_id: {axis: {"magnitude": float, "enacted_tick": int, "promised": float, "delivered": float}}}`) | `magnitude ∈ [0,1]` (upstream `PolicyAgendaItem.magnitude` constraint, `policy.py:98`; **not enforced** at this read site — `_border_valve` re-clamps defensively, line 147) | **untyped side-channel register — no BSL structural equivalent (§6)** |
| CLAIMS edge `control_level` | edge attr (sovereign→territory) | `float` (`topology/graph.py:954`) | unconstrained at the read site (`float(data.get("control_level", 0.0))`) | **edge attribute — no BSL read lane exists (§6)** |
| `sigmoid_k` (define) | — | `float` | `(0.0, 100.0]` | bounded-above real coefficient |
| `sigmoid_r0` (define) | — | `float` | `(0.0, 1.0]` | unit-interval coefficient |
| `wage_pressure_ceiling` (define) | — | `float` | `(0.0, 1.0]` | unit-interval coefficient |
| `min_employed_fraction`, `mechanization_displacement_rate`, `firm_failure_conversion_rate` (defines) | — | `float` | `[0,1]` / **`(0, ∞)` unbounded above** / `(0,1]` respectively (`economy_labor.py:99-151`) | consumed by the **cross-system** `DefaultAccumulationLoopCalculator`, not by this system's own call graph (§5) |

**"Currency" flag (Territory's precedent applies identically).** `median_wage` is `Currency`-typed
in the Python reference but the Python "Currency" is an unbounded-above plain float, not BSL's
`i128` type; `GraphSubstrate` attribute storage is always plain `f64`, and CURRENT BSL surface
confirms Currency-typed field *storage* is refused. `median_wage` is not itself involved in any
`Currency × Ratio`-class BSL scale-op question here — the multiply `median_wage * (1.0 -
wage_pressure)` (Step 4) is `Currency × Real` in Python-reference terms; the same bare-scaled-Int
workaround precedent (Metabolism D-1, Territory's `rent_level` finding) applies straightforwardly
IF `median_wage` is declared `int` per the landed-pack convention — this is a workaround, not a
blocker.

**No enum discriminant in this system.** Unlike Territory (`profile`/`territory_type`), every
attribute `ReserveArmySystem` reads or writes is a plain float/dict — there is no `deffield enum`
gap here.

**The genuinely novel type-shape gap: an untyped graph-level side-channel dict, and an edge
attribute read.** `policy_overlays` is not a per-node attribute at all — it is a
`graph.get_graph_attr`/`set_graph_attr` metadata register (`graph_protocol.py:350-368`), a
mechanism BSL's node/edge model has no equivalent for. Even setting that aside, reading
`control_level` off a CLAIMS edge to rank claims-holders needs an `EdgeRef` attribute read —
explicitly Slice 2+, not landed (see CURRENT BSL surface). Both facts are catalogued precisely
in §6.

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `float`). Two `math.exp` calls — the system's only
libm hazard, and its only hazard of any kind. No `math.log`/`math.pow`/`math.sqrt` anywhere in
`reserve_army.py` or `calculator.py` (grep-confirmed). Shapes, in execution order:

1. **Guard comparisons:** `reserve_ratio <= 0.0` (×3: `reserve_army.py:80,90`; `calculator.py:41`), `wage_pressure <= 0.0` (`reserve_army.py:95`), `current_wage > 0.0` (`reserve_army.py:105`), `max_raw <= 0.0` (`calculator.py:61`).
2. **Multiplicative throttle:** `reserve_ratio *= 1.0 - border` (`reserve_army.py:89`) — one subtract, one multiply, same "`x * (1 - rate)`" idiom Territory's heat-decay used.
3. **Nested clamp, mixed nesting order (`_border_valve`):** `min(1.0, max(0.0, float(magnitude)))` (`reserve_army.py:147`) — clamps the *lower* bound first (inner `max`), then the *upper* (outer `min`).
4. **Sigmoid core — LIBM HAZARD #1 & #2:** `raw = 1.0 / (1.0 + math.exp(exponent))` (`calculator.py:52`) and `baseline = 1.0 / (1.0 + math.exp(baseline_exponent))` (`calculator.py:57`). `math.exp` is a libm transcendental; cross-language/cross-implementation bit-identity is **not** guaranteed by IEEE-754 the way `+ − × ÷` and comparison are (§4.3's basic-op list, per `bsl-language.rst:3196-3199` — transcendentals cross via a pinned soft-float libm crate with golden vectors, a different, heavier mechanism). This is flagged as a determinism hazard **independently of** the ADR188 doctrine-legality blocker in §6 — even if this formula were re-derived as a measure rather than a stipulated sigmoid, any future `exp` use anywhere in the estate inherits the same r21 golden-vector obligation.
5. **Exponent pre-clamp, nested min-then-max:** `exponent = max(min(exponent, 500.0), -500.0)` (`calculator.py:51`) and the identical shape at `calculator.py:56` — clamps the *upper* bound first (inner `min`), then the *lower* (outer `max`). **Structurally the reverse nesting order from item 3's `_border_valve` clamp**, even though both clamp to a symmetric range and both are mathematically a plain two-sided clamp — a genuine syntactic inconsistency (semantically equivalent, differently expressed), the same class of finding Territory's inventory flagged for its two `heat` clamps, though here neither instance is *behaviorally* different (Territory's was upper-only vs. full-range; here both are full-range, only the nesting/argument order differs).
6. **Final normalize clamp:** `ceiling * max(0.0, min(normalized, 1.0))` (`calculator.py:65`) — same min-then-max nesting as item 5 (`max(A, B)` vs. `max(B, A)`-order swap only — cosmetically different from item 5, not a genuine inconsistency).
7. **Redundant literal arithmetic, transcribed as-is (port-as-is note):** `baseline_exponent = -k * (0.0 - r0)` (`calculator.py:55`) computes `-k * -r0` via an explicit `0.0 - r0` subtraction rather than a bare negation `-r0`; mathematically identical, a verbatim quirk to preserve if this is ever ported, not a defect to silently simplify.
8. **Wage reduction:** `float(current_wage) * (1.0 - wage_pressure)` (`reserve_army.py:106`) — one subtract, one multiply, same idiom as item 2.

**Real→Int demotions in this system: none.** No `int(...)` cast appears anywhere in
`reserve_army.py` or `calculator.py` (grep-confirmed; the only `int`/`float` usage is
`isinstance(x, (int, float))` type-narrowing checks and `float(x)` up-casts). This differs from
the cross-system accumulation-loop producer (`accumulation.py:115,121`), which uses Python's
`round()` — **not** `int()`/truncation — on `delta_occ * employment * rate` and `bankruptcy_rate
* employment * rate` respectively. `round()` is banker's-rounding (round-half-to-even), a
**different** semantic from the `floor` intrinsic ADR188 Row 2 landed (`floor` ≡ truncation for
non-negative operands, per `declarations.rs:3357-3371`); `round-half-even` is a **named,
separately-tracked gap** — "obliged by §3.2/§2.7... still sits outside the enumeration" (Row 3,
`bsl-language.rst:3207-3208, 3269-3273`; "housekeeping rider" ratified by ADR188 but "landing
each one's own normative text... is separate work this revision does not perform"). This finding
belongs to `TickDynamicsSystem`'s own port inventory (out of scope here — the calculator is not
invoked by `ReserveArmySystem.step()`), recorded in §5 for completeness only.

**Bare non-integer literals requiring BSL's `c`-suffix treatment (a parser-level fact, per
Territory's precedent):** `1.0` (8 occurrences across 6 lines, grep-confirmed:
`reserve_army.py:89,106,147`; `calculator.py:52`×2`,57,60,65`), `0.0` (×5+, guards, defaults, and
`calculator.py:55`'s `0.0 - r0`), `500.0`/`-500.0` (`calculator.py:51,56` — the exponent-overflow
clamp bound, a magic number with no `defines` backing at all — see §6).

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 5.0** (`reserve_army.py:37`), confirmed against `_SYSTEM_CLASSES`
  (`simulation_engine.py:328-364`): `VitalitySystem(1.0) → TerritorySystem(2.0) →
  SubstrateSystem(2.5) → ProductionSystem(3.0) → TickDynamicsSystem(4.0) →
  ReserveArmySystem(5.0) → CommunitySystem(6.0) → ...`. Positions verified directly:
  `vitality.py:80`, `territory.py:59`, `substrate.py:162`, `production.py:68`,
  `tick/system/__init__.py:124`, `reserve_army.py:37`, `community.py:319`.

- **Reads from a same-tick prior system — two real channels, one false friend:**
  1. **`TERRITORY.reserve_ratio` from `TickDynamicsSystem` (@4.0), same tick.**
     `TickDynamicsSystem._compute_accumulation_loop` (`tick/system/__init__.py:1243-1334`)
     derives `ReserveArmyDynamics` from an organic-composition delta
     (`ValueTensor4x3.organic_composition`, via `services.tensor_registry`) and a FRED
     bankruptcy rate (`services.dispossession_data_source`), accumulates it into
     `TERRITORY.reserve_army_stock`, and writes `TERRITORY.reserve_ratio` directly onto the
     graph (`tick/system/__init__.py:1318-1323`) — explicitly documented as feeding
     `ReserveArmySystem` "later in this SAME tick" (docstring, `:1263-1265`). **Gated:** this
     entire annual pipeline (`TickDynamicsSystem.step()`) only executes `if tick %
     WEEKS_PER_YEAR == 0` (`:174`, a year-boundary gate) and only if `services.melt_calculator
     is not None` (`:198-200`); the accumulation loop itself additionally no-ops if both
     `tensor_registry` and `dispossession_data_source` are `None` (`:1287-1288`).
  2. **`TERRITORY.reserve_ratio` from `MarketScissorsSystem` (@17.8), prior tick.** On a rare
     price/value-correction "snap" (`market_scissors.py:338-386`, gated on unserviceable
     overhang + a cooldown), `_swell_reserve_army` (`:439-460`) bumps `reserve_ratio` on active
     territories carrying a `median_wage`: `graph.update_node(node.id,
     reserve_ratio=min(base + influx, 1.0))` (`:460`). Since MarketScissors runs at 17.8 (after
     ReserveArmySystem's 5.0), this write is consumed **next tick** by ReserveArmySystem's
     ordinary read — the docstring says so explicitly ("The @5 system converts the ratio into
     wage pressure NEXT tick", `:446`).
  3. **False friend — `reserve_army_signal()` does NOT feed this system.**
     `domain/economics/tick/graph_bridge.py:511-541` computes a differently-named,
     differently-sourced "downturn signal" (`s_r`, from county-level U-3 unemployment, for the
     endogenous-interest-rate credit machinery) that the function's own docstring states reads
     `county_states`, "never `TERRITORY.reserve_ratio`" in substance — "`CountyEconomicState`
     carries no `reserve_ratio`; U-3 is its labor-slack field" (`:522-523`). It shares only the
     name "reserve army" with this system; grep-confirmed zero read/write overlap.

- **Special-note correction — this system does NOT consume the `market_correction_shock`
  register.** `MARKET_CORRECTION_SHOCK_ATTR = "market_correction_shock"`
  (`wealth_distribution.py:77`) is stamped by the **same** MarketScissors snap event that
  triggers `_swell_reserve_army` above — all three calls are adjacent in one atomic block
  (`market_scissors.py:384-386`: `_evaporate_wealth`, `_swell_reserve_army`,
  `graph.set_graph_attr(MARKET_CORRECTION_SHOCK_ATTR, ...)`) — but the register itself is
  **consumed only by `WealthDistributionSystem`** (`_consume_market_shock`,
  `wealth_distribution.py:135-153`); grep across `src/babylon/engine/systems/` confirms zero
  other readers. `ReserveArmySystem` consumes a **sibling** write from the same snap event (the
  `reserve_ratio` bump above), not the shock register itself. Recorded as a correction to this
  inventory's originating special note, not silently absorbed.

- **A second, independent wage-pressure application exists elsewhere in the engine, using the
  SAME calculator class on a DIFFERENT data source and target field.**
  `TickDynamicsSystem._compute_vol1_layer` / `._compute_vol1_county_state`
  (`tick/system/__init__.py:1175-1241`) also instantiates `DefaultWagePressureCalculator`
  (`:1199`) and calls `wage_calc.compute_wage_pressure(state.reserve_ratio)` (`:1239`) — but
  `state` here comes from `services.reserve_army_data_source.get_unemployment_decomposition(fips,
  year)` (`:1235`, the SQLite-backed `ReserveArmyState.reserve_ratio` *property*,
  `total_reserve/labor_force`, `types.py:44-46`), an entirely separate data path from the
  graph-attribute `reserve_ratio` this system reads, and it writes `CountyEconomicState.median_wage`
  (`:1241`), a different field from `Territory.median_wage`. Both call sites are the "wage-pressure
  sigmoid" ADR188 Row 7 names (one shared function, two independent callers) — a port of this
  formula must account for both, though only the first (`ReserveArmySystem`) is this inventory's
  target.

- **Writes consumed downstream:**
  - `TERRITORY.median_wage` (this system's write, Step 4) — read next tick by
    `MarketScissorsSystem._swell_reserve_army`'s active-territory filter
    (`market_scissors.py:455-456`, "the same attribute `ReserveArmySystem` discounts"). No other
    System in `src/babylon/engine/systems/` reads `median_wage` (grep-confirmed).
  - `TERRITORY.wage_pressure` — read by **no other System** (grep-confirmed); it is dropped from
    the model on every `WorldState.from_graph()` (§3) and its only downstream consumers are the
    event mirror (Step 5) and the seam/observer projection layer
    (`sentinels/seam/registry.py:385-397,1216-1226` — a rendering surface, not engine logic).
    Terminal/observational output.
  - `TERRITORY.reserve_ratio` (re-stamped by Step 4 even on the un-throttled path) — read next
    tick by this same system and by `MarketScissorsSystem._swell_reserve_army`'s accumulation
    (`market_scissors.py:458-460`, `current = attrs.get("reserve_ratio")`).
  - `EventType.RESERVE_ARMY_PRESSURE` — no in-engine `System` subscriber (grep-confirmed);
    consumed by `chronicle_adapter.py:187-191` (narrative text rendering) and
    `event_builders.py:483-490` (typed reconstruction for observers). Per the CURRENT BSL
    surface, `TickReport` carries no event log — this emission is a WS1 (#502) ledger item,
    unpinnable by goldens today, same as every other `EventType` finding in this program.

- **Context/service usage with no BSL equivalent:** none beyond the graph-level
  `policy_overlays` register already catalogued in §2/§3 (`protocol.get_graph_attr(...)`,
  `reserve_army.py:67`). `context.tick` (`TickContext.tick`, `engine/context.py:48`) is read only
  to stamp the emitted event (Step 5) — an ordinary scalar, no BSL gap.

- **Dormancy — declared, not merely inferred.** `tools/regression_scenarios.py:2802-2808`
  (`COVERAGE_GAPS_DATA`) states explicitly: *"no territory carries a positive reserve_ratio in
  any of the five scenarios; RESERVE_ARMY_PRESSURE never fires and median_wage/wage_pressure are
  territory-scoped"* — remediation: "seed reserve_ratio on a canonical scenario's territories."
  This is corroborated three ways: (1) `reserve_army.py:66`'s own comment — "Absent register ⟹
  identical math — the qa six never carry it" (re: `policy_overlays`); (2) `CHANNEL_WRITERS`
  (`tools/regression_scenarios.py:2880-2925`) — a maintained registry of which System writes
  which named channel across the canonical suite — has **no entry at all** for `reserve_ratio`,
  `median_wage`, or `wage_pressure`, consistent with the channel never firing on canonical data;
  (3) direct test evidence: `tests/unit/engine/systems/test_feature021_territory_roundtrip.py:9-13`
  states outright "no production seeder writes `reserve_ratio` / `foreclosure_rate` /
  `median_wage` onto territory nodes... these tests seed the inputs explicitly." The mechanism
  that COULD produce a live `reserve_ratio` (`TickDynamicsSystem`'s accumulation loop) is wired
  with real calculators in `qa:regression` (`services.py:162-256`, `_build_vol3_melt_calculator`
  + a real-FIPS `tensor_registry`), but the coverage-gap text is unambiguous that it never
  produces a positive value across the canonical suite regardless. The `border_regime` valve path
  (§2 Step 2) is separately dormant: it needs a `CLAIMS` edge and a `policy_overlays` register no
  canonical scenario seeds (only the hand-built fixture in `test_policy.py:402-420` exercises it).

## 6. BLOCKER ASSESSMENT

| Computation | Verdict | Detail |
|---|---|---|
| Step 1 — territory iteration + reserve_ratio gate (`reserve_army.py:71-81`) | **PORTABLE NOW** | `nodes`/typed iteration + a plain `:field` read + a guard comparison — Query lane Slice 1 (landed, ADR197) covers this exactly, matching Territory's own precedent for `:field`-sourced reads. |
| Step 2 — border-regime valve (`reserve_army.py:83-91`, `_border_valve` 123-147) | **BLOCKED — two missing lanes, not one** | (a) Ranking CLAIMS-holders by `control_level` needs an **edge-attribute read** (`EdgeRef`, explicitly Slice 2+, NOT landed per the CURRENT BSL surface — "no edge-attribute reads (EdgeRef)"). (b) Even with edge attributes, `policy_overlays` is a graph-level, untyped, arbitrarily-nested Python dict (`{sovereign_id:{axis:{...}}}`) — BSL's node/edge model has **no construct for a free-form graph-side-channel register at all**; representing it would require either promoting the register to first-class per-node fields on `sovereign` nodes (itself contingent on `PolicySystem`'s own port — a large, unstarted system) or some other design not yet proposed anywhere in the estate. Dormant on every canonical scenario (§5), so no conformance oracle exists either way today. |
| Step 3 — wage-pressure sigmoid (`calculator.py:32-65`) | **BLOCKED — RULED, ADR188 Row 7 (not an open question)** | The Director has already ruled (2026-08-10, `ai/decisions/ADR188_intrinsic_rider_slate_dispositions.yaml`) that "the wage-pressure sigmoid" — named explicitly, this exact function — must "RE-DERIVE AS MEASURES at the port... the S-curve emerges from within-class dispersion," and that "No BSL rule may ever stipulate a logistic form." Mechanically enforced two ways: `sigmoid` is a `PROHIBITED_INTRINSIC_NAME` (`declarations.rs:116`, `E-LOAD-024`), and even a hand-assembled equivalent using the declarable `exp` intrinsic would "pass the cap check and violate the theory line" (`bsl-language.rst:3210-3217` — cap-legality ≠ doctrine-legality; gate 2 is "not mechanical... belongs to Director review"). **This is not a missing-grammar gap a D-record can paper over** — the redesign (what measure replaces it) is itself undone port-time design work, per ADR188's own consequences: "becomes a port-time design obligation on their systems' BSL rule packs... each carries its derivation note at the port" (ADR188 `consequences`). Also independently a libm-nondeterminism site (`math.exp` ×2) even setting the doctrine question aside — see §4 item 4. |
| Step 4 — wage multiply + node write (`reserve_army.py:98-107`) | **PORTABLE NOW, contingent on Step 3** | Plain `Currency-as-int`-workaround multiply (`x * (1 - pressure)`, same idiom as Territory's heat decay) + `update-node` against the currently-iterated node (no computed `NodeRef` needed — landed). The `wage_pressure`/`reserve_ratio` re-stamp needs no field-storage widening since both are already `[0,1]`-domain floats with landed precedent. Blocked only in the sense that it has nothing to write until Step 3 produces a value. |
| Step 5 — event emission (`reserve_army.py:109-121`) | **PORTABLE WITH D-RECORD** | `emit` exists in BSL, but per the CURRENT BSL surface `TickReport` carries no event log — this is a WS1 (#502) ledger row (the same disposition every other `EventType` finding in this program gets), unpinnable by goldens today, not a grammar blocker. |
| `policy_overlays` register representation (cross-cutting, Step 2) | **BLOCKED — same missing lane as Step 2, named separately for clarity** | See Step 2. This is the same underlying gap Territory's own report never encountered (Territory reads no graph-level side-channel register) — a genuinely new finding for this port program. |
| `TickContext.tick` read (event stamp only) | **PORTABLE NOW** | An ordinary scalar; no BSL gap. |

**Summary disposition.** Of the system's five steps, three (1, 4, 5) are mechanically portable
today or with a routine D-record; one (Step 2, the border valve) is blocked on a named missing
BSL lane (edge-attribute reads) **plus** an unrepresented graph-side-channel construct, and is
also structurally dormant on every canonical scenario; and the system's actual reason for
existing — Step 3, the wage-pressure sigmoid — is not blocked by a missing feature but by a
**ratified Director prohibition** with unperformed redesign work. A pack that ported Steps 1/4/5
around a stubbed-out Step 3 would compute nothing real; this is the "sliver-only port... rejected
as silent scope shrink" pattern Territory's own inventory named, in a more acute form (here the
sliver *is* the whole system minus its one purpose).

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_reserve_army_system.py` | 229 | **Primary conformance oracle.** Exercises Steps 1/4/5 end-to-end via `ReserveArmySystem().step()`: wage-pressure application, monotonicity vs. ratio, zero-ratio/no-ratio no-ops, non-territory skip, `wage_pressure` node storage, event publish/no-publish, multi-territory processing, ceiling-boundedness at an extreme ratio (0.99). **Does not exercise Step 2 (border valve) at all** — no `policy_overlays`/CLAIMS setup anywhere in the file (confirmed by full read). |
| `tests/unit/engine/laws/test_law_reserve_army.py` | 209 | **Property-based invariant contracts** (hypothesis, `max_examples=25`): L1 inactivity-on-zero-ratio, L2 wage-pressure bounded `[0,ceiling]` + wage-stays-positive, L3 ratio-monotonicity (weak `≥`, since the sigmoid saturates), L4 event/node payload agreement. The file's own header docstring is itself a precise, load-bearing spec of the calculator's behavior (`:1-42`) — a strong conformance-oracle candidate for any BSL re-derivation, since these are *behavioral* laws (bounds, monotonicity, agreement) rather than bit-exact-formula pins, and would still need to hold for whatever measure replaces the sigmoid. |
| `tests/unit/engine/systems/test_policy.py:402-420` | 728 (whole file) | **The only test exercising the border-regime valve** (`test_border_regime_throttles_the_reserve_army_valve`) — hand-seeds `policy_overlays` directly via `graph.set_graph_attr`, since no canonical scenario or factory produces one. A conformance-oracle candidate for Step 2 specifically, though it lives in `PolicySystem`'s test file, not this system's own. |
| `tests/unit/economics/reserve_army/test_calculator.py` | 94 | `DefaultWagePressureCalculator` unit tests in isolation: zero/negative ratio, monotonicity, ceiling saturation, midpoint behavior, custom defines, overflow-safety at ratio=1.0, small-ratio near-zero, output-range sweep, two named Wayne/Oakland County scenarios. The single richest bit-behavior oracle for Step 3 — but every assertion here is a fact about *this specific sigmoid formula*, which ADR188 rules must not be the formula that ships; these tests pin an implementation the port is directed to replace, not one to reproduce. |
| `tests/unit/economics/reserve_army/test_accumulation.py` | 281 | `DefaultAccumulationLoopCalculator` unit tests — **out of scope** (cross-system producer, §5), not this system's own conformance surface. |
| `tests/unit/economics/reserve_army/test_types.py` | 198 | `ReserveArmyState`/`ReserveArmyDynamics` Pydantic model validation — schema-level, and for the SQLite-backed data path (§5's "second application"), not this system. |
| `tests/unit/engine/systems/test_feature021_territory_roundtrip.py` | 121 | **Round-trip/wiring conformance.** Confirms the lowercase `_node_type="territory"` match (the historical case-bug fix), confirms `wage_pressure` is dropped on `from_graph()` while `median_wage` persists, and a 3-tick `simulation_engine.step()` loop confirming compounding wage suppression across ticks (`:110-121`) — a genuine multi-tick behavioral contract, useful for a port's own multi-tick conformance vector. |
| `tests/unit/engine/test_system_order.py` | 300 | Confirms position 5.0 and name `"reserve_army"` — ordering/schema-level, not behavior. |
| `tests/unit/test_public_import_surface.py` | 309 | Confirms `ReserveArmyDefines` is publicly importable — schema-level. |
| `tests/unit/sentinels/test_seam_liveness.py` | 398 | Confirms `wage_pressure`'s `DECLARED_CONDITIONAL` seam-registry classification (line ~232) — observer/vocabulary governance, not tick-behavior conformance. |
| `tests/unit/models/test_event_severity.py`, `tests/unit/engine/test_event_conversion.py` | 644, 1774 | Generic per-`EventType` severity/conversion sweeps that happen to include `RESERVE_ARMY_PRESSURE` alongside all ~100 other event types — not system-specific. |

**qa:regression byte-gate coverage.** As with Territory, `tools/regression_test.py`'s
`graph_content_hash` hashes every node/edge attribute on every canonical scenario, so any change
to this system's outputs *would* be caught — but per §5's declared dormancy finding, that
coverage is vacuous for this system today: no canonical scenario ever produces a positive
`reserve_ratio`, so `ReserveArmySystem` computes nothing on any byte-gated run. A port's
conformance fixtures must be hand-built (mirroring `test_policy.py`'s and
`test_feature021_territory_roundtrip.py`'s own precedent of explicit `reserve_ratio`/
`policy_overlays` seeding), not harvested from the canonical estate.

---

## Adjudication (2026-08-12)

Adjudicated against the **current dev tree** (`9324482f`) by a second, read-only
pass, in the manner of the Territory inventory's own "Adjudicated verdict"
section. Two corrections, five confirmations. The core verdict survives intact;
its dormancy premise does not.

1. **CONFIRMATION — ADR188 Row 7 verified verbatim, and this report's framing is
   the correct one for the whole batch.**
   `ai/decisions/ADR188_intrinsic_rider_slate_dispositions.yaml:54-57` reads
   *"Row 7  exp — the three stipulated-sigmoid sites (P(S|A) survival calculus,
   the defection probability, the wage-pressure sigmoid) RE-DERIVE AS MEASURES
   at the port — the S-curve …"*, with the standing ruling restated at `:10`.
   `rust/crates/babylon-bsl/src/declarations.rs:116` —
   `PROHIBITED_INTRINSIC_NAMES: [&str; 1] = ["sigmoid"]`, enforced at `:684`;
   `DECLARABLE_INTRINSICS` at `:110` is `["exp", "log", "floor"]`, so a
   hand-assembled equivalent is cap-legal and doctrine-illegal exactly as §6
   argues. **This is a closed ruling with undone design work, not an open
   escalation** — and it is the framing the TickDynamicsSystem inventory in this
   same batch gets wrong for the *same* `DefaultWagePressureCalculator`
   (`src/babylon/domain/economics/reserve_army/calculator.py:32-65`, verified
   line-by-line against §2 Step 3's transcription, including the redundant
   `0.0 - r0` at `:55`). Reconcile that report to this one, not the reverse.
2. **CORRECTION — the "dormant on every canonical scenario" premise rests on a
   stale source and a misread corroborator; two of its three legs do not hold.**
   - *Leg (a), stale.* The `COVERAGE_GAPS_DATA` row quoted
     (`tools/regression_scenarios.py:2801-2807`) says *"no territory carries a
     positive reserve_ratio in any of the **five** scenarios"* — the ORIGINAL
     FIVE, written before the U13/ADR140 electoral goldens joined `SCENARIOS`,
     which now has **12** keys (`:37-133`). Six of those twelve carry
     `county_fips="26163"` (`src/babylon/engine/scenarios/single_county.py:116`;
     `electoral_goldens.py:225` mitterrand, `:270` syriza, `:331-332` weimar,
     `:423-424` debs, `:498` bernie_valve), and four —
     `WAYNE_CALCULATOR_SCENARIOS` (`:151-153`) — take
     `build_single_county_overrides`, i.e. a real Wayne `tensor_registry`
     (`tools/regression_test.py:1030-1034`). The producer of `reserve_ratio`,
     `TickDynamicsSystem._compute_accumulation_loop`, returns early **only when
     `tensor_registry` AND `dispossession_source` are both `None`**
     (`src/babylon/domain/economics/tick/system/__init__.py:1285-1288`) — not
     when either is — and the annual gate (`:174`) fires at tick 52 of a 52-tick
     run (`tools/regression_test.py:81`, `:1054`). So the mechanism §5 concedes
     "COULD produce a live `reserve_ratio`" **does execute** on four canonical
     scenarios; whether `compute_dynamics` returns non-`None` there (it needs a
     positive OCC delta, since `dispossession_data_source` is unwired) is
     **UNVERIFIED** and is what a re-read must settle by a spot-run, not by
     citation.
   - *Leg (b), misread.* `CHANNEL_WRITERS` is not a registry of "which System
     writes which named channel across the canonical suite". Its own header
     (`tools/regression_scenarios.py:2870-2879`) reads *"Dense-column suffix ->
     System class names that may write it"* — its keys are the pinned dense-trace
     COLUMN suffixes. `reserve_ratio`/`median_wage`/`wage_pressure` are
     territory-scoped and simply are not dense columns (the dense trace pins
     entity and edge fields, `tools/regression_test.py:422`), so their absence
     is evidence of nothing about liveness.
   - *Leg (c), survives.*
     `tests/unit/engine/systems/test_feature021_territory_roundtrip.py:9-13`
     stands as quoted — but it speaks to SEEDING, not to the TickDynamics-derived
     path, which is exactly the path leg (a) reopens.

   The system may still be dormant; the report has not established it.
3. **CORRECTION — no RESERVED-LINE finding is recorded**
   (`grep -c -i "RESERVED-LINE"` → 0), and this system carries one in its own
   source comment. `src/babylon/engine/systems/reserve_army.py:86-88` annotates
   the border-valve throttle as *"A tighter border throttles the reserve army's
   replenishment: the effective ratio shrinks, wage pressure eases — the
   settler-wing wage bargain."* That is the imperial-bribe / National Question
   surface ADR171 reserves to the Director (MIM+MLP line, B+C+I partition,
   bribe:deprivation = 1.55). A port transcribes the valve's *direction* (tighter
   border ⟹ less reserve army ⟹ higher wages for the protected wing) and its
   magnitude verbatim; **which wing the bribe protects, and whether the valve
   exists at all, is not a port-time transcription call.** The re-read owes an
   explicit RESERVED-LINE section saying so.
4. **CONFIRMATION — the border valve's two blockers are correctly and
   separately named.** (a) `BabylonGraph.query_territory_claims`
   (`src/babylon/topology/graph.py:942-959`) ranks CLAIMS rows by the **edge
   attribute** `control_level`; `edges`, `edge-between` and `the` are all in
   `UNSERVED_EXPRESSION_HEADS` under **slice 2**
   (`rust/crates/babylon-bsl/src/evaluator.rs:504-506`). (b) `policy_overlays`
   is a `get_graph_attr` register (`reserve_army.py:67`) and there is **no
   graph-level attribute construct anywhere in either Rust crate** (`rg -n
   "graph_attr|graph_attribute|GraphAttr" rust/crates/babylon-graph/src/
   rust/crates/babylon-bsl/src/` → 0 hits). Added teeth the report does not have:
   the R9 chapter-C3 escape hatch for graph-scope state is itself unusable here,
   since it prescribes `(field-of (the NodeType/…) …)` /
   `(update-node (the NodeType/…) …)`
   (`docs/reference/bsl-language.rst:2650-2669`) and `the` is the same slice-2
   head. **Both blockers stand, one of them harder than stated.**
5. **CONFIRMATION — Step 4's presence/type guard is genuinely portable despite
   the no-`bound?` rule, and the report is right not to have flagged it — but the
   D-record it implies is missing.** `docs/reference/bsl-language.rst:2628-2633`
   (read directly) rules `:optional` requires `:default` and *"there is
   consequently no `bound?` predicate in the language"*. The frozen guard is
   `isinstance(...) and current_wage > 0.0` (`reserve_army.py:105`), so an
   absent-defaulted `0.0` and a present `0.0` take the SAME branch — behaviourally
   identical, no gap. The one transcribable difference is that the frozen code
   omits the `median_wage` KEY entirely in that branch, whereas a BSL node always
   carries a declared field's value. That is a D-record row the blocker table
   does not have; add it.
6. **CONFIRMATION — this system dodges D102, one of only two in this batch.**
   `field-of` naming an `:enum-type`-declared field is REFUSED AT LOAD
   (`rust/crates/babylon-bsl/src/typecheck.rs:266-289`,
   `check_no_field_of_on_enum_field`). §3's "no enum discriminant in this
   system" finding means no fold body in a ReserveArmy pack ever needs an
   enum-valued `field-of` — a favourable structural fact this report identified
   without yet knowing it was load-bearing (the Production, TickDynamics and
   Community inventories in this batch are all bitten by it).
7. **CONFIRMATION — tick position 5.0 and the channel table's load-bearing
   writes.** `src/babylon/engine/systems/reserve_army.py:37`
   (`position: ClassVar[float] = 5.0`) against the 34-member `_SYSTEM_CLASSES`
   (`src/babylon/engine/simulation_engine.py:328-363`; order derived by sorting
   on `position`, `:376-377`), with the neighbours re-verified directly
   (`vitality.py:80` 1.0, `territory.py:59` 2.0, `substrate.py:162` 2.5,
   `production.py:68` 3.0, `tick/system/__init__.py:124` 4.0, `community.py:319`
   6.0). Spot-checks: `rg -ln "median_wage" src/babylon/engine/systems/` →
   `market_scissors.py`, `reserve_army.py` exactly; `rg -ln "reserve_ratio"` →
   the same two; `wage_pressure` has no reader in `systems/` at all. Every
   channel claim in §5 holds.

**FINAL VERDICT: BLOCKED — CONFIRMED on both substantive counts, with the
dormancy premise withdrawn pending re-derivation.** Step 3 (the wage-pressure
sigmoid) is blocked by a **ratified Director prohibition with unperformed
redesign work** (ADR188 Row 7, verified verbatim), not by a missing grammar —
and this report's insistence on that distinction is the correct reading for the
whole batch. Step 2 (the border valve) is blocked twice over: the CLAIMS
`control_level` read needs the slice-2 edge-attribute lane, and `policy_overlays`
has no BSL representation at all, with C3's carrier-node escape hatch itself
gated behind the same slice-2 `the`. Steps 1/4/5 are mechanically portable and
the sliver-only argument stands. **But "dormant on every canonical scenario
today" is not established** — the row it rests on says "the five scenarios"
against a 12-scenario estate, and the accumulation-loop producer does execute on
four of them. Whether it produces a positive `reserve_ratio` is the open
question, and it changes only the conformance-oracle picture, not the two
blockers.

**COVERAGE NOTE (not inadequacy).** The module-level read is thorough and the
blocker table is right where it matters. A re-read owes exactly three additions:
(i) a spot-run-backed dormancy re-derivation across the six county-bearing
scenarios replacing the stale five-scenario citation and dropping the
`CHANNEL_WRITERS` leg (correction 2); (ii) the RESERVED-LINE section
(correction 3); (iii) the absent-`median_wage` D-record row (confirmation 5).
