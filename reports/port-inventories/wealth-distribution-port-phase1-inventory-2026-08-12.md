# WealthDistributionSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `WealthDistributionSystem` (position 21.5, 274 lines,
`src/babylon/engine/systems/wealth_distribution.py`) is a Program 21 Phase-1 SHADOW: it
seeds/advances a single national 4-bracket wealth-share vector (a second-order ODE,
`formulas/class_dynamics.py`) and projects it onto each `social_class` node's `wealth_share`
field. Every piece of its state — the vector itself, its ODE velocities, the one-tick-lagged
market-correction shock it consumes from `MarketScissorsSystem` @17.8 — lives in
`graph.graph[...]` metadata, not on any node, which is exactly the Q6 "graph-scope state" gap
`bsl-language.rst` §3.6 already RULES a fix for (a `:ceiling 1` carrier node read/written via
`the`) but that primitive is `UNSERVED_EXPRESSION_HEADS`-refused today (Slice 2, not built,
`evaluator.rs:506`) — so the system has no BSL storage home at all, seed step included.
Independently, the per-bracket aggregation (`_bracket_resistances`, and the final
role→bracket node-projection loop) needs to partition `social_class` nodes by an 8-valued
`role` enum, which needs `field-of` on the iterated element — refused for enum-declared
fields by D102 — so even a hypothetical storage fix would not unblock those two computations.
A third, independently-confirmed finding: `_bracket_resistances` reads a flat
`node.attributes["class_consciousness"]` key that no production graph ever carries (the real
value lives nested at `ideology.class_consciousness`, written by `ConsciousnessSystem` @17.0)
— a pre-registered, task-#45-audited LIVE BUG (`sentinels/vocabulary/registry.py:433-449`)
that makes the ODE's resistance term provably always `0.0` in every real game.

**Verdict:** BLOCKED — the whole system is graph-scope state with no BSL storage primitive
today (Q6/`the`, Slice 2 unbuilt), and its role-partitioned aggregation is separately blocked
by the enum-`field-of`-on-`it` refusal (D102); the underlying scalar arithmetic (Euler step,
seed normalization, the market-shock impulse) is trivial and D-record-ready once a storage
home lands, and the frozen system's own resistance input is provably dead code today
regardless.

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/wealth_distribution.py` | 274 | **The target.** `WealthDistributionSystem.step()` (218-274) plus module-level helpers `bracket_of_role` (80-86), `_seed_vector` (89-102), `_ode_params` (105-132), `_consume_market_shock` (135-158), `_bracket_resistances` (161-180), `_advance` (183-215), and the `_BRACKET_BY_ROLE` mapping (59-68, "RATIFIED owner ruling 2026-07-16, ADR075"). |
| `src/babylon/formulas/class_dynamics.py` | 357 | The wired ODE. Called by `step()`: `calculate_class_dynamics_derivative` (137-198, first-order flows) and `calculate_wealth_acceleration` (201-228, second-order momentum, via `calculate_full_dynamics` 231-277). **Not called by `step()`** (re-exported in `formulas/__init__.py` but zero production callers, grep-confirmed): `calculate_wealth_flow` (113-134), `calculate_equilibrium_deviation` (280-303, test-only), `invert_wealth_to_population` (306-357) — dead from this system's perspective. |
| `src/babylon/config/defines/economy_class.py` | 322 | `ClassDynamicsDefines` (12-179) — the 16-field coefficient model (`alpha_*`/`delta_*`/`gamma_3`/`beta_*`/`omega_*`/`equilibrium_w*`/`ticks_per_quarter`), all `Field(ge=..., le=...)` bounded. |
| `src/babylon/config/defines/market.py` | 245 | `MarketDefines.wealth_axis_kick_gain` (203-213, `[0.0, 0.1]`, default `0.02`) — the ONLY define this system reads outside `class_dynamics`. |
| `src/babylon/data/defines.yaml` | `class_dynamics:` block 458-481; `wealth_axis_kick_gain` line 1009 | Player-editable coefficient values. |
| `src/babylon/models/wealth_distribution.py` | 52 | `WealthDistribution` — the graph-metadata carrier's Pydantic shape (`shares`/`velocities`/`tick`), `model_validator` enforcing `Σshares == 1 ± 1e-6` and each share in `[0,1]` (38-52). |
| `src/babylon/models/entities/social_class.py` | 522 | `SocialClass.wealth_share` (312-322, `Probability`, declared field — the `extra="forbid"` landmine precedent) and `SocialClass.ideology: IdeologicalProfile` (335-338) — the REAL nested home of `class_consciousness` (see §3/§4 bug finding). |
| `src/babylon/models/enums/social.py` | 211 | `SocialRole` (12-43, `StrEnum`, 8 members) + `.coerce` (45-64, string/member/`None` coercion used by both `_bracket_resistances` and the projection loop). |
| `src/babylon/models/enums/topology.py` | 253 | `NodeType.SOCIAL_CLASS = "social_class"` (62). |
| `src/babylon/kernel/node_access.py` | 37 | `class_consciousness_from_node` (15-35) — the canonical nested-`ideology`-dict reader EVERY OTHER consumer of this value uses (`struggle.py`, `solidarity.py`, `economic.py`, `epistemic_horizon.py`). `wealth_distribution.py` does not import this helper — the root cause of the §3/§4 bug. |
| `src/babylon/engine/systems/ideology.py` | 442 | `ConsciousnessSystem` (position 17.0) — the same-tick prior producer of `ideology` (`graph.update_node(node.id, ideology={"class_consciousness": ..., ...})`, 418-424). |
| `src/babylon/engine/systems/market_scissors.py` | 582 | `MarketScissorsSystem` (position 17.8) — writes `MARKET_CORRECTION_SHOCK_ATTR` graph metadata (386, same-tick producer §5) AND imports `bracket_of_role` from `wealth_distribution.py` (39, 431) for its own claim-holder classification — a two-way, same-pair cross-system dependency (see §5). |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase` — `step()` uses NONE of its helpers (`_write_clamped`/`_publish`/`_read`/`_wrap_graph` — grep-confirmed zero `self.` references anywhere in the file); every clamp is hand-inlined. |
| `src/babylon/kernel/system_protocol.py` | 41 | `ContextType = "TickContext"` (16, string-literal type alias). |
| `src/babylon/kernel/tick_partition.py` | 30 | `TickPartition.CONSEQUENCE` (18-28). |
| `src/babylon/kernel/graph_protocol.py` | 494 | `GraphProtocol` — declares `get_graph_attr`/`set_graph_attr` (350-366) and `query_nodes`/`update_node` (77-88, 258-266) but **no `.graph` property at all** — `step()` bypasses the declared accessors entirely via `getattr(graph, "graph", None)` (see §5). |
| `src/babylon/topology/graph.py` | 1033 | Concrete `BabylonGraph.graph` property (338-340, "nx-style graph-level attribute dict (live)", returns `self._graph_attrs` directly) and `.set_graph_attr`/`.get_graph_attr` (896-898, 892-894) — the SAME dict reached two ways; `step()` and `_consume_market_shock` use the raw-dict path exclusively, never the protocol methods. `update_node` (660) is a plain dict merge, no mid-tick quantization (same fact the Territory inventory records). |
| `src/babylon/engine/context.py` | 113 | `TickContext.tick: int = 0` (48) — already `int`, so `int(tick)` at `wealth_distribution.py:267` is a no-op cast, not a Real→Int demotion. |
| `src/babylon/engine/simulation_engine.py` | 611 | `_SYSTEM_CLASSES` (328-362) — confirms tick position 21.5, second-to-last of 34 (only `EpistemicHorizonSystem` @22.0 follows). |
| `src/babylon/sentinels/vocabulary/registry.py` | 754 | `ATTRIBUTE_EXEMPTIONS` row (433-449) — the PRE-REGISTERED, task-#45-audited record of the flat-`class_consciousness`-key bug this inventory independently re-derives (§3/§4). |
| `tools/regression_scenarios.py` | — | `SCENARIO_COVERAGE_DATA` — two claims naming `WealthDistributionSystem`/`C001.wealth_share` (389-394, 803-807) plus one naming `MarketScissorsSystem`/`market_correction` in the SAME coverage block (367-373, `imperial_circuit` scenario) — the cross-system channel is live on canonical data (§5). |

**Not exercised by `step()` at all:** `formulas/class_dynamics.py`'s `calculate_wealth_flow`,
`calculate_equilibrium_deviation`, `invert_wealth_to_population` (test-only/orphaned, see
above); `SystemBase`'s clamp/read/publish helpers; `GraphProtocol.get_graph_attr`/
`set_graph_attr` (bypassed in favor of the raw `.graph` dict).

**Reference BSL packs and spec sections read for this inventory** (all read directly, not
cited secondhand): `metabolism.bsl` (full, 413 lines — D-record conventions, the
Real-zero-promotion trick, the bare-scaled-Int workaround, the `(domain :graph)`-not-executed
D-4 finding); `docs/reference/bsl-language.rst` §2.4-2.6 (Conditions/Bindings/Queries,
694-1020), §2.13 (the enum `deffield` row and its `field-of` deferral, 2255-2284), §3.4 (the
intensivity kind rule and the `E-TYPE-042` unweighted-intensive-mean refusal, 2532-2601), §3.6
(closed vocabulary + the R9 chapter C3 graph-scope-state ruling naming "the national wealth
vector" by name, 2639-2689); `rust/crates/babylon-bsl/src/evaluator.rs` (`UNSERVED_EXPRESSION_
HEADS`, 486-512 — confirms `the` is Slice-2-refused on the CURRENT checkout, not merely per a
possibly-stale citation); `rust/crates/babylon-bsl/src/tick.rs` and
`rust/crates/babylon-tick/src/lib.rs` (grepped directly for `RuleDomain::Graph`/`loaded.domain`
— zero hits in either, confirming `(domain :graph)` still does not execute on dev HEAD);
`rust/crates/babylon-bsl/src/declarations.rs`/`types.rs`/`structural_verbs.rs` (confirms
`defenum`/`EnumRegistry`/`Value::Enum` ARE landed on dev HEAD, past the ADR195/196 YAML
records' own "NOT YET DONE" language — see the §6 note); `reports/bsl-gap-analysis-2026-08-10.md`
(row 79, and the Q6/Q7/Class-E sections, 243-289, 668-671) — an independent, one-day-earlier
survey that reaches the same "BSL_RULES (blocked)" disposition this inventory reaches by its
own, separately-verified route.

## 2. COMPUTATION CATALOG (execution order, `step()` at `wealth_distribution.py:234-274`)

### Step 0 — Read the prior vector or detect first tick (`step`, 246-255)
- **(a)** Look up `graph.graph["wealth_distribution"]`; its absence means "first tick," which
  seeds from the calibration defines instead of advancing.
- **(b)** `metadata = getattr(graph, "graph", None)`; `if not isinstance(metadata, dict): return`
  (248-250, a defensive no-op guard the docstring calls unreachable in production — `# pragma:
  no cover`); `prior = metadata.get("wealth_distribution")` (251).
- **(c) Reads:** `graph.graph` (the raw nx-style metadata dict, NOT `GraphProtocol.get_graph_attr`).
- **(d) Writes:** none yet.
- **(e) Defines:** none.
- **(f) Events:** none.

### Step 1a — First-tick seed (`_seed_vector`, 89-102; `step`, 253-255)
- **(a)** Normalize the four `equilibrium_w1..w4` calibration defines so they sum to exactly 1,
  and seed velocities at zero — then still apply any pending market-shock impulse to that
  zero-velocity vector.
- **(b)** `raw = (w1, w2, w3, w4)`; `total = sum(raw)`; `return (raw[0]/total, ..., raw[3]/total)`
  (95-102). `shares = _seed_vector(defines)`; `velocities = _consume_market_shock(metadata,
  (0.0,0.0,0.0,0.0), kick_gain)` (254-255).
- **(c) Reads:** `services.defines.class_dynamics.equilibrium_w1..w4`.
- **(d) Writes:** none directly (feeds Step 3).
- **(e) Defines:** `class_dynamics.equilibrium_w1..w4` (each `[0,1]`, defines.yaml:462-465) — **note:
  the four raw defaults sum to `1.001`, not exactly `1.0`** (`0.305+0.382+0.294+0.02`), so this
  normalization is load-bearing, not decorative — it is what lets `WealthDistribution`'s own
  `Σ==1±1e-6` validator (`models/wealth_distribution.py:49-51`) pass at all. `tests/unit/config/
  test_wealth_distribution_invariants.py::test_shares_sum_to_one` only asserts `abs(total-1.0)
  <= 0.01`, i.e. it does NOT require the raw defines to sum exactly to 1 — confirming the
  runtime normalization is doing real work, not merely defending against float noise.
- **(f) Events:** none.

### Step 1b — Market-correction shock consumption (`_consume_market_shock`, 135-158; called from both branches, 255/257)
- **(a)** If `MarketScissorsSystem` stamped a correction THIS tick, apply a conservation-preserving
  velocity impulse: bracket 0 (top-1%) loses `kick`, the other three each gain `kick/3` — then
  clear the stamp so it fires exactly once.
- **(b)** `shock = metadata.pop(MARKET_CORRECTION_SHOCK_ATTR, None)`; `if shock is None: return
  velocities` (150-152); `kick = kick_gain * float(shock["overhang"])` (153); `if kick == 0.0:
  return velocities` (154); `third = kick/3.0` (157); `return (v1-kick, v2+third, v3+third,
  v4+third)` (158).
- **(c) Reads:** `graph.graph["market_correction_shock"]` (a transient dict `{"tick": int,
  "overhang": float}`, written same-tick by `MarketScissorsSystem`); `services.defines.market.
  wealth_axis_kick_gain`.
- **(d) Writes:** pops (deletes) `graph.graph["market_correction_shock"]` — a same-tick,
  read-once-then-clear consumption pattern (the "malformed stamp fails LOUD" docstring claim at
  138-143 is aspirational for a malformed `shock["overhang"]` KeyError, not defended by any
  try/except in this function — a `KeyError` would propagate, which IS loud, just undocumented
  as a specific guard).
- **(e) Defines:** `market.wealth_axis_kick_gain` (`[0.0, 0.1]`, default `0.02`, `market.py:203-213`,
  defines.yaml:1009).
- **(f) Events:** none.

### Step 2 — Per-bracket resistance aggregation (`_bracket_resistances`, 161-180; called only on the advance branch, 261)
- **(a)** Compute the mean `class_consciousness` of each of the four wealth brackets, to feed the
  ODE's resistance term (organized classes resist extraction). **This computation is provably
  dead in every real game — see the §3/§4 bug finding below.**
- **(b)** `nodes = sorted(graph.query_nodes(node_type=NodeType.SOCIAL_CLASS), key=lambda n: n.id)`
  (172); for each node, `role = SocialRole.coerce(node.attributes.get("role"))`, skip if `None`
  (174-176); `bracket = _BRACKET_BY_ROLE[role]`; `sums[bracket] += float(node.attributes.get(
  "class_consciousness", 0.0))`; `counts[bracket] += 1` (177-179); `return tuple(s/c if c else
  0.0 for s, c in zip(sums, counts))` (180).
- **(c) Reads:** `TERRITORY`... no — `SOCIAL_CLASS.role` (enum, via `.coerce`); `SOCIAL_CLASS.
  class_consciousness` — **a flat top-level key that no production graph ever carries** (see
  below).
- **(d) Writes:** none (pure aggregation feeding Step 3).
- **(e) Defines:** none.
- **(f) Events:** none.
- **THE BUG (independently re-derived, then found pre-registered).** `ConsciousnessSystem`
  @17.0 (a same-tick PRIOR system) writes the value as a NESTED dict field:
  `graph.update_node(node.id, ideology={"class_consciousness": new_class, "national_identity":
  new_nation, "agitation": new_agitation})` (`ideology.py:418-424`). Every other consumer in the
  codebase reads it through the canonical nested accessor `class_consciousness_from_node`
  (`node_access.py:15-35`, `ideology = node_data.get("ideology"); ideology.get(
  "class_consciousness", 0.0)`) or an equivalent manual `ideology.get(...)` (`struggle.py:96`,
  `solidarity.py`, `electoral.py:404`, `allegiance.py:290`, `epistemic_horizon.py:96`).
  `_bracket_resistances` is the ONLY reader in the codebase that does `node.attributes.get(
  "class_consciousness", 0.0)` FLAT — a key that is never written by any production system, so
  this line always returns the default `0.0` for every node, meaning `_bracket_resistances`
  ALWAYS returns `(0.0, 0.0, 0.0, 0.0)` on any real graph. This is PRE-REGISTERED as a known live
  bug: `sentinels/vocabulary/registry.py:433-449` (`ATTRIBUTE_EXEMPTIONS`, key `("node_attribute",
  "tests/unit/engine/test_wealth_distribution_system.py", "social_class", "class_consciousness")`,
  reason: *"wealth_distribution.py::_bracket_resistances reads a flat 'class_consciousness' key
  that NO production graph carries ... the ODE's resistance term is silently always 0.0 in every
  real game," tracking_task="#45"*. The system's own test fixture
  (`tests/unit/engine/test_wealth_distribution_system.py:62`, `_graph_with_classes()`) stamps
  `class_consciousness=0.2` FLAT on synthetic nodes, which is why every existing test passes
  despite the bug — the fixture's shape matches the buggy read, not real production shape.
  **Downstream consequence:** in `calculate_class_dynamics_derivative` (§4), every `(1 - r_i)`
  resistance-damping factor is therefore ALWAYS exactly `1.0` in production — the resistance
  mechanism is completely inert, a decorative no-op, regardless of any class's actual
  consciousness level.

### Step 3 — One Euler step of the ODE (`_advance`, 183-215; `step`, 258-263)
- **(a)** Advance the national vector one tick: integrate acceleration into velocity, integrate
  velocity+flow into shares, clamp each share to `[0,1]`, then renormalize the whole vector so
  it sums to exactly 1 (conservation of the whole — float drift never accumulates).
- **(b)** `dt = 1.0 / defines.ticks_per_quarter` (203); `params, second_order = _ode_params(defines)`
  (204, builds `ClassDynamicsParams`/`SecondOrderParams` from the SESSION's defines, not
  import-time frozen defaults — 105-132); `flows, accelerations = calculate_full_dynamics(shares,
  velocities, params=params, second_order=second_order, resistances=resistances)` (205-207);
  `new_velocities = tuple(v + a*dt for v, a in zip(velocities, accelerations))` (208); `stepped =
  [max(0.0, min(1.0, w + (f+v)*dt)) for w, f, v in zip(shares, flows, new_velocities)]` (209-212);
  `total = sum(stepped)`; `normalized = tuple(w/total for w in stepped) if total > 0.0 else
  _seed_vector(defines)` (213-214).
- **(c) Reads:** `prior["shares"]`/`prior["velocities"]` (from graph metadata); `_bracket_
  resistances(graph)` output (always zero, see Step 2); `services.defines.class_dynamics.*`
  (all 16 fields).
- **(d) Writes:** none directly (returns `(normalized, new_velocities)`, written in Step 4).
- **(e) Defines:** `alpha_21/31/32/41/42/43` (each `[0, 0.01]`), `delta_1/2/3` (each `[0, 0.1]`),
  `gamma_3` (`[0, 0.1]`), `beta_1..4` (each `[-1.0, 0.0]` — a bounded-NEGATIVE domain, unusual
  among this codebase's coefficients), `omega_1..4` (each `(0.0, 1.0]`, strictly positive),
  `ticks_per_quarter` (`[1.0, 91.0]`) — all `economy_class.py:21-179`, defines.yaml:459-481.
- **(f) Events:** none.

### Step 4 — Write the advanced/seeded vector to graph metadata (`step`, 264-268)
- **(a)** Persist the (seeded or advanced) national vector back to graph metadata for next tick.
- **(b)** `metadata["wealth_distribution"] = {"shares": list(shares), "velocities":
  list(velocities), "tick": int(tick)}` (264-268) — direct dict-item assignment, not
  `graph.set_graph_attr(...)`.
- **(c) Reads:** `context.tick` (already `int`, `TickContext.tick: int = 0`).
- **(d) Writes:** `graph.graph["wealth_distribution"]` (dict; round-trips through `WorldState.
  wealth_distribution: WealthDistribution | None`, `world_state.py:507-516`, `to_graph()`
  888-889, `from_graph()` 1057-1062 — written ONLY when set, so axis-less graphs stay
  byte-identical, matching the EH ruling-6 precedent).
- **(e) Defines:** none.
- **(f) Events:** none.

### Step 5 — Per-node bracket projection (`step`, 269-274)
- **(a)** Every `SOCIAL_CLASS` node with a recognized role gets its own bracket's current wealth
  share written onto it, so per-class code can read a class's national wealth position without
  re-deriving the bracket fold.
- **(b)** `nodes = sorted(graph.query_nodes(node_type=NodeType.SOCIAL_CLASS), key=lambda n:
  n.id)` (269); for each, `role = SocialRole.coerce(node.attributes.get("role"))`, `continue` if
  `None` — "honest absence: no role, no bracket, no projection" (271-273, the code's own
  comment); `graph.update_node(node.id, wealth_share=shares[_BRACKET_BY_ROLE[role]])` (274).
- **(c) Reads:** `SOCIAL_CLASS.role` (enum).
- **(d) Writes:** `SOCIAL_CLASS.wealth_share` (`Probability [0,1]`, declared field,
  `social_class.py:312-322`).
- **(e) Defines:** none directly (consumes Step 3/4's output).
- **(f) Events:** none.

**Events emitted by the whole system: zero.** Grep-confirmed — no `EventType`/`_publish`/
`.emit(` reference anywhere in `wealth_distribution.py` or `formulas/class_dynamics.py`.

## 3. TYPE INVENTORY

| Attribute | Node type | Python model type | Domain | Category |
|---|---|---|---|---|
| `role` | SOCIAL_CLASS | `SocialRole` (StrEnum, 8 members) | closed set | **Enum discriminant** |
| `class_consciousness` (flat key, as READ by this system) | SOCIAL_CLASS | — (not a real declared field at this path) | n/a | **Dead/buggy read — see §2 Step 2.** The REAL field is `ideology: IdeologicalProfile` (nested `BaseModel`), whose `class_consciousness` sub-field is `Annotated[float, ge=0.0, le=1.0]` (`social_class.py:83-90`). |
| `wealth_share` | SOCIAL_CLASS | `Probability` | `[0.0, 1.0]` | unit-interval, declared field, write-only from this system |
| `graph.graph["wealth_distribution"].shares` | — (graph-scope, not node-scope) | `tuple[float,float,float,float]` (`WealthDistribution.shares`) | each `[0,1]`, Σ==1±1e-6 (model-validated) | unit-interval vector, conservation-constrained |
| `graph.graph["wealth_distribution"].velocities` | — (graph-scope) | `tuple[float,float,float,float]` | **UNBOUNDED reals — no `Field()` constraint at all** (`models/wealth_distribution.py:35`) | unbounded real vector |
| `graph.graph["wealth_distribution"].tick` | — (graph-scope) | `int` | `≥ 0` | integer |
| `graph.graph["market_correction_shock"]` | — (graph-scope, transient, popped same-tick read) | `dict` `{"tick": int, "overhang": float}` | `overhang` domain defined by the PRODUCER (`MarketScissorsSystem`), not bounded within this system's own code | producer-defined, out of this system's scope |
| `equilibrium_w1..w4` (defines) | — | `float` | each `[0.0, 1.0]`, sum ≈ `1.001` raw (not exactly 1 — see §2) | unit-interval coefficients |
| `alpha_21/31/32/41/42/43` (defines) | — | `float` | each `[0.0, 0.01]` | small-positive-rate coefficients |
| `delta_1/2/3` (defines) | — | `float` | each `[0.0, 0.1]` | small-positive-rate coefficients |
| `gamma_3` (defines) | — | `float` | `[0.0, 0.1]` | small-positive-rate coefficient (constant injection into `dw3`) |
| `beta_1..4` (defines) | — | `float` | each `[-1.0, 0.0]` | **bounded-NEGATIVE domain** — unusual, damping-only, never positive |
| `omega_1..4` (defines) | — | `float` | each `(0.0, 1.0]` strictly positive | oscillation-frequency coefficients, squared in Step 3 (see §4) |
| `ticks_per_quarter` (define) | — | `float` | `[1.0, 91.0]` | divisor (`dt = 1.0/ticks_per_quarter`) |
| `wealth_axis_kick_gain` (define) | — | `float` | `[0.0, 0.1]`, default `0.02` | unit-interval-adjacent coefficient |

**No Currency-typed value anywhere in this system.** Unlike Territory (`rent_level`) or
Metabolism (`entropy_factor`), every scalar this system reads or writes is a plain bounded
`float`, `Probability`, or unbounded-real — the `#500` Currency-scale-op / ADR183 bare-scaled-Int
workaround class simply does not apply here; there is no `Currency x Ratio`/`Real x Ratio`
hazard to route around.

**Enum discriminant, same landed-lane status as every other current-dev system.** `role`
(8-valued `SocialRole`) is a `deffield ... enum SocialRole` candidate under the NOW-LANDED enum
lane (ADR195/196 — see §6's verification note that the Rust runtime, not just the language spec,
is confirmed live on dev HEAD). Storage is not the gap; **reading `it`'s enum value inside a
query body IS** — see §6.

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `float`). Shapes, in execution order:

1. **Seed normalization** (`_seed_vector`, 95-102): `total = sum(raw)` (3 adds); `raw[i]/total`
   ×4 (division).
2. **Market-shock impulse** (`_consume_market_shock`, 153-158): `kick = kick_gain * float(...)`
   (multiply); `kick == 0.0` (bare-float equality comparison — safe here, since `kick` is exactly
   `0.0` only when one factor is exactly `0.0`, not the product of two independently-rounded
   nonzero values); `third = kick / 3.0` (division by bare literal `3.0`); `v_i ± kick`/`± third`
   ×4 (add/subtract).
3. **Resistance mean** (`_bracket_resistances`, 178-180): accumulation (`sums[bracket] +=`);
   conditional division `s/c if c else 0.0` (guards div-by-zero). **Dead in practice — §2/§3.**
4. **`dt` divisor** (`_advance`, 203): `1.0 / defines.ticks_per_quarter` — bare literal `1.0`.
5. **First-order flows** (`calculate_class_dynamics_derivative`, `class_dynamics.py:171-198`):
   ~11 multiply/add/subtract terms across `dw1`/`dw2`/`dw3`; each resistance factor is `(1 -
   r_Y)` where the literal `1` is a bare Python **int**, not `1.0` (auto-promotes on subtraction
   with a float — a minor style inconsistency, not a bug, but worth noting: BSL's "no bare
   non-integer literal" rule would need this written as an explicit Real-typed `1c`/promotion,
   same idiom `metabolism.bsl` already uses); `dw4 = -dw1 - dw2 - dw3` (negation + 2 subtracts).
6. **Second-order acceleration — the POW-shaped hazard** (`calculate_wealth_acceleration`,
   `class_dynamics.py:228`): `damping * velocity - (frequency**2) * (wealth_share -
   equilibrium)`. **`frequency**2` is syntactically a `pow` operation** — Python's `**` operator
   for two floats is, in the general case, CPython's `float.__pow__`, which the task's own
   framing asks to flag as a libm-shaped nondeterminism hazard alongside `exp`/`log`/`sigmoid`.
   I could not inspect CPython's C source from this sandbox to confirm whether it special-cases
   an integer literal exponent of exactly `2` to a plain multiply internally (UNVERIFIED —
   CPython internals, no source available here) — flagged regardless because the REFERENCE
   Python source is syntactically a `**` call, which a byte-for-byte transcription would need to
   reproduce. **Mitigation available to a port:** since the exponent is the literal integer `2`,
   this is trivially reformulable as `(mul frequency frequency)` — an ordinary multiply, not a
   `pow`/`exp`/`log` intrinsic call (BSL's `DECLARABLE_INTRINSICS` is only `exp`/`log`/`floor`
   anyway — there is no `pow` intrinsic to invoke even if one wanted to transcribe the Python
   `**` literally). This is the ONLY libm-shaped operation anywhere in this system or its formula
   module — grep-confirmed zero `exp`/`log`/`sigmoid`/general `pow(` calls otherwise.
7. **Velocity integration** (`_advance`, 208): `v + a*dt` ×4 (multiply-add).
8. **Share integration + clamp — a THIRD distinct clamp shape** (`_advance`, 209-212): `max(0.0,
   min(1.0, w + (f+v)*dt))` ×4 — an inline, per-element, doubly-nested `max(lo, min(hi, ...))`
   clamp, immediately followed by a renormalizing division (item 9). This is neither Territory's
   `_write_clamped` helper NOR Territory's hand-written upper-only `min(1.0, ...)` — it is its
   own third shape (lower-AND-upper, hand-inlined, list-comprehension body), worth its own
   D-record note if the pack is ever built, though notably a WELL-BEHAVED design: clamping each
   term individually before summing, then dividing by the actual (post-clamp) sum, guarantees
   every final share stays in `[0,1]` by construction (each term is ≥0 and ≤ the sum of all four
   nonnegative terms) — no separate defensive re-clamp is needed after the divide.
9. **Renormalization** (`_advance`, 213-214): `total = sum(stepped)` (3 adds); conditional
   `w/total if total > 0.0 else _seed_vector(...)` ×4 (division, bare literal `0.0` guard,
   fallback to a fresh re-seed on the degenerate all-zero case).
10. **`int(tick)` cast** (`step`, 267): a no-op — `tick` is already `TickContext.tick: int`, so
    this is NOT a Real→Int demotion (unlike Territory's two genuine truncating casts).

**No `floor`/truncation anywhere; no genuine Real→Int demotion.** This is a materially cleaner
float surface than Territory's — the only flagged hazard is the single `**2` (item 6), and it is
resolvable by reformulation rather than by needing a new declared intrinsic.

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 21.5** (`wealth_distribution.py:229`), confirmed against `_SYSTEM_CLASSES`
  (`simulation_engine.py:328-362`): `... EdgeTransitionSystem (21.0) → WealthDistributionSystem
  (21.5) → EpistemicHorizonSystem (22.0)`. Second-to-last of 34.
- **Reads from a same-tick prior system — TWO real channels (unlike Territory's "none"):**
  1. `ConsciousnessSystem` @17.0 writes `SOCIAL_CLASS.ideology` (nested dict, `ideology.py:
     418-424`) — the value `_bracket_resistances` SHOULD read but does not, due to the flat-key
     bug (§2/§3). The channel exists in the graph; the read side is broken.
  2. `MarketScissorsSystem` @17.8 writes `graph.graph["market_correction_shock"]`
     (`market_scissors.py:386`) — genuinely read AND consumed (popped) same-tick by
     `_consume_market_shock` (`wealth_distribution.py:150-158`). **This channel is live and
     exercised on canonical data** (see the dormancy note below) — a materially different fact
     from Territory, whose only cross-system input was a hardcoded-`EXTRACTION` test-only
     context override.
- **A reverse, import-time dependency (not a graph-state channel, but a real coupling):**
  `MarketScissorsSystem` (`market_scissors.py:39, 431`) imports `bracket_of_role` FROM
  `wealth_distribution.py` to classify which `SOCIAL_CLASS` nodes are "claim holders"
  (`_CLAIM_HOLDER_BRACKETS = (0, 1)`, `market_scissors.py:66-68`) whose fictitious wealth
  evaporates during a correction. The module docstring explains why `MARKET_CORRECTION_SHOCK_ATTR`
  is declared in `wealth_distribution.py` (the CONSUMER) rather than `market_scissors.py` (the
  PRODUCER): "@17.8 already imports this module for the bracket fold — the reverse import would
  cycle" (`wealth_distribution.py:72-76`). Net effect: the `_BRACKET_BY_ROLE` classification
  table is doing double duty across two systems, which matters for any eventual port — a shared
  BSL content source (not two independent transcriptions) would need to serve both packs.
- **Downstream reads of this system's writes: ZERO.** Grep-confirmed across every file in
  `src/babylon/engine/systems/*.py`, plus `src/babylon/intelligence/`, `src/babylon/projection/`,
  and the legacy `web/` client: no System, no AI/narrative layer, and no projection reads
  `wealth_share` or `graph.graph["wealth_distribution"]`. `EpistemicHorizonSystem` @22.0 is the
  only System that runs after this one, and it does not reference "wealth" anywhere in its own
  file (grep-confirmed). This matches the module's own docstring claim exactly ("Nothing reads
  these outputs yet"). `formulas/class_dynamics.py`'s own `wealth_share`/`wealth_shares`
  parameter names are internal to the formula module, not a cross-system read.
- **Context/service usage:** `services.defines.class_dynamics` (16-field `ClassDynamicsDefines`)
  and `services.defines.market.wealth_axis_kick_gain` — both ordinary `:const`-shaped defines
  reads with no BSL representation problem in isolation. `context.tick` maps cleanly to the
  `:tick` bind-src.
- **The load-bearing architectural fact: `step()` bypasses `GraphProtocol` entirely for
  graph-scope reads/writes.** `GraphProtocol` (the declared abstract interface systems are
  supposed to code against) exposes `get_graph_attr`/`set_graph_attr` (`graph_protocol.py:
  350-366`) — but has **no `.graph` property at all**. `step()` instead does `metadata =
  getattr(graph, "graph", None)` (248) and mutates the returned dict directly
  (`metadata["wealth_distribution"] = {...}`, `metadata.pop(MARKET_CORRECTION_SHOCK_ATTR, None)`)
  — reaching `BabylonGraph`'s own concrete `.graph` property (`topology/graph.py:338-340`, "the
  SAME dict as `set_graph_attr` reaches, exposed a second, undeclared way") rather than the
  protocol's own accessor methods. This is the clearest evidence in the Python codebase that
  "graph-scope state" is a real, load-bearing category the `GraphProtocol` abstraction does not
  even attempt to cover formally — independently corroborating the §6 Q6 finding below at the
  Python-architecture level, not just the BSL-language level.
- **Dormancy on canonical scenarios — LIVE, unlike Territory.** `tools/regression_scenarios.py`'s
  `SCENARIO_COVERAGE_DATA` names `WealthDistributionSystem`/`C001.wealth_share` as an
  `entity_delta` claim in TWO scenario coverage blocks (389-394, 803-807) — the seed/advance/
  projection path runs and produces observable per-node output on canonical data (every
  canonical scenario seeds `SOCIAL_CLASS` nodes with a `role`, which is all the projection loop
  needs). The SAME coverage block that carries the `WealthDistributionSystem` claim (lines
  360-394, scenario `"imperial_circuit"`) ALSO carries a `MarketScissorsSystem`/`"market_
  correction"` `event` claim (367-373) — confirming the cross-system shock-consumption channel
  is exercised live on at least one canonical scenario, not merely a hand-built fixture concern.
  **What IS provably dead on every canonical run regardless:** the `_bracket_resistances` term
  (§2/§3's bug) — since it always returns `(0,0,0,0)` in production, no canonical scenario, no
  matter how it is seeded, will ever exercise a nonzero-resistance code path; a conformance
  fixture harvested from live data would silently inherit the same blind spot the bug already
  creates.

## 6. BLOCKER ASSESSMENT (adjudicated against the CURRENT BSL surface, dev HEAD 2026-08-12)

**Verification note on the task's enum-lane framing.** The ADR195/ADR196 YAML records
themselves say the Rust runtime is "NOT YET DONE" (Tasks 3-10 of a separate plan) — read alone,
this would make the task's "Enum fields LANDED... STALE" framing look wrong. Checking the
CURRENT dev tree directly resolves this: `defenum`/`EnumRegistry`/`Value::Enum` read+write paths
ARE implemented (`declarations.rs:587` `parse_defenum`; `types.rs:108` `struct EnumRegistry`;
`tick.rs:374` `Value::Enum{...}`; `structural_verbs.rs:1251` the write-side type check) and a
real golden (`rust/crates/babylon-tick/content/scenarios/organization-foundation.bscn`) exists —
confirming Tasks 3-10 landed in the commits AFTER the ADR YAMLs were written (`git log`: PR #534,
#549, #550 land between ADR196 and this inventory). The task's framing is correct for dev HEAD;
the ADR YAMLs are an accurate snapshot of an EARLIER point the tree has since moved past.

| Computation | Verdict | Detail |
|---|---|---|
| Seed/read graph-scope state (`step` Step 0/1a, reading and about to write `graph.graph["wealth_distribution"]`) | **BLOCKED — Q6, graph-scope state, no storage primitive today** | `bsl-language.rst` §3.6 (2650-2689) RULES this exact case — it names "the national wealth vector" BY NAME among the 22 systems the ruling covers — and specifies the mechanism: a `:ceiling 1` carrier `NodeType`, read via `(field-of (the NodeType/…) …)`, written via `(update-node (the NodeType/…) …)`. The primitive the ruling depends on, `the`, is in `UNSERVED_EXPRESSION_HEADS` (`evaluator.rs:506`, `("the", "slice 2")`) — confirmed directly against `tick.rs`/`lib.rs` (zero `RuleDomain::Graph`/`loaded.domain` references in either) that `(domain :graph)` execution and `the` evaluation are BOTH still absent on dev HEAD. The ruling exists at the spec level; nothing in the evaluator serves it yet. |
| Market-shock consumption (`_consume_market_shock`, reading `graph.graph["market_correction_shock"]` and popping it) | **BLOCKED — same Q6 gap, plus no `<bind-src>` for a raw dict key at all** | Beyond the storage-home gap above: `<bind-src>` closes at exactly 8 forms (`:field`/`:const`/`:metric`/`:tick`/`:year`/`:tick-of-year`/`:tick-in-cycle`/`:expr`, §2.5, 827-834) — none names an arbitrary graph-metadata dict key, and no §2.8 verb writes or DELETES one either (the "pop" semantics — read-once-then-clear — has no verb-level analogue at all, carrier-node route or not). Even once a carrier node existed for the wealth vector, "consume and clear a transient stamp written by a DIFFERENT system's carrier" is a second, un-ruled cross-carrier interaction. |
| Per-bracket resistance aggregation (`_bracket_resistances`) | **BLOCKED — D102, enum `field-of` on the iterated element (`it`), independent of Q6** | Partitioning `(nodes NodeType/SOCIAL_CLASS)` by an 8-valued `role` enum needs SOME way to read `it`'s `role` inside a `<node-pred>`/fold body. `field-of` is the accessor for exactly this shape (`(field-of it social-class/role)`) but is explicitly REFUSED for `:enum-type`-declared fields (D102, `bsl-language.rst:2273-2284`, confirmed live in `typecheck.rs:246-280` and `tick.rs:848,1304-1335` — both cite "D102" in their refusal messages). The only landed enum-read path is a `:field` binding, which binds against the RULE's own `self` (§2.5), not an iterated `it` inside a query body — so there is no way to filter or group a node-set query by role today. **Independently, even a resolved D102 would not make this computation's EXACT frozen arithmetic expressible**: `class_consciousness` is the spec's own worked example of an `:kind intensive` field ("`consciousness` (Intensity) is intensive", `bsl-language.rst:2537`), and `fold mean` over an intensive body with no `:weight` is `E-TYPE-042` (2547, 2579-2584) — the frozen Python's UNWEIGHTED per-bracket mean is exactly the shape BSL's own type system structurally refuses, independent of the query-lane gap. (Moot in practice today, since the flat-key bug makes every summed term `0.0` regardless of weighting — see §2/§3 — but real once that bug is ever repaired.) |
| First/second-order ODE arithmetic itself (`calculate_class_dynamics_derivative`, `calculate_wealth_acceleration`, the Euler integration in `_advance`) | **PORTABLE WITH D-RECORD (transitively, once a storage home exists)** | Pure scalar multiply/add/subtract over `:const`-sourced coefficients, all within declared `[lo,hi]` domains — trivial `defconst` transcription, same class as every landed pack. The one flagged item, `frequency**2` (§4 item 6), reformulates cleanly as `(mul frequency frequency)` — no new intrinsic needed. This row is arithmetically ready; it is blocked only because it has nowhere to read its `shares`/`velocities` operands FROM or write its result TO (the Q6 row above). |
| Seed normalization (`_seed_vector`) | **PORTABLE WITH D-RECORD (transitively)** | `raw[i]/total` over four `:const` defines — trivial arithmetic. Blocked only by the same Q6 storage-home gap; the write TARGET, not the math, is missing. |
| Per-node bracket projection (`step` Step 5, `graph.update_node(node.id, wealth_share=shares[bracket])`) | **BLOCKED — D102, same enum-comparison gap, PLUS a second-order Q6 dependency** | Determining each node's bracket needs the SAME role-enum read D102 refuses. Even setting that aside, the projected VALUE (`shares[bracket]`) is itself graph-scope state (Q6) with no representable read source today — this row is doubly blocked, not merely transitively. |
| `_BRACKET_BY_ROLE`/`bracket_of_role` classification table (owner-ratified content, ADR075) | **PORTABLE WITH D-RECORD, once D102 is resolved** | A `defenum SocialRole (...)` plus an explicit chain of `=` comparisons (matching the market-scissors sibling inventory's own treatment of the identical table) is a legitimate content-modeling transcription once SOME enum-comparison-on-`it` mechanism lands — not a language gap in the mapping itself, just downstream of the same D102 blocker every other role-partitioned computation in this system hits. |
| `EventType` emissions | **N/A — zero emitted** | Grep-confirmed no `EventType`/`_publish`/`.emit(` reference anywhere in this system or its formula module (§2). No WS1 (#502) ledger row is owed here, unlike every other system in this port-estate survey that emits at least one event. |

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/test_wealth_distribution_system.py` | 265 | **Primary conformance-oracle candidate.** Seeding-matches-equilibrium, per-node bracket projection, share-conservation-every-tick, perturbation mean-reversion, determinism, the market-shock impulse (deflate-top-bracket / conserve-the-whole / consume-once / no-stamp-is-identical), the `WorldState` round-trip (including the golden-preservation "absent axis writes no metadata key" law), bracket-mapping completeness. **Its own fixture (`_graph_with_classes`, line 62) stamps `class_consciousness` FLAT — reproducing the §2/§3 bug's exact wrong shape, not real production shape** — any conformance fixture built FROM this test file's pattern would need to be corrected to nested `ideology={"class_consciousness": ...}` to test the SYSTEM's real behavior rather than the bug's self-consistent shadow. |
| `tests/unit/formulas/test_class_dynamics.py` | 767 | Pure formula-module unit tests — exercises ALL SEVEN `class_dynamics.py` functions, including the three this system's `step()` never calls (`calculate_wealth_flow`, `calculate_equilibrium_deviation`, `invert_wealth_to_population`, §1). A conformance-oracle candidate for the ARITHMETIC only, not for the system's graph-integration behavior — and it tests dead code alongside live code without distinguishing them. |
| `tests/unit/config/test_wealth_distribution_invariants.py` | 139 | **RESERVED-LINE adjacent** (see below) — pins `equilibrium_w1..w4` inside empirically-sourced WID/Piketty ⋃ Fed-DFA bands, and asserts "structural wealth laws" framed explicitly as "the Fundamental Theorem in empirical form." A calibration/schema test, not a system-behavior conformance oracle — describes what the shipped defines MUST be, not what `WealthDistributionSystem` computes. |
| `tests/unit/reference/test_fred_wealth_shares.py` | 121 | Redundant-source corroboration against `fact_fred_wealth_shares` in the reference SQLite DB. Tests reference DATA, not this system's code at all — zero relevance to a port's conformance surface beyond corroborating the calibration defines' provenance. |
| `tests/unit/engine/test_system_order.py` | 300 | Pins `WealthDistributionSystem` at position 21.5 in the full 34-system ordering (line 96, 190, 265). An ORDERING conformance oracle, not a computation one. |
| ~~`tests/unit/economics/melt/test_class_position.py`~~ | 773 | **FALSE POSITIVE from an initial name-collision grep — excluded.** Tests `DefaultWealthProxyCalculator.classify_wealth_distribution` (`domain/economics/melt/wealth_proxy.py`), an entirely unrelated per-county population→class classifier that happens to share the substring "wealth_distribution" in a method name. Confirmed by import inspection (`from babylon.domain.economics.melt import (...)`, `wealth_proxy import DefaultWealthProxyCalculator`) — zero references to `formulas.class_dynamics` or `engine.systems.wealth_distribution` anywhere in the file. |
| ~~`tests/unit/economics/melt/test_sc002_measurability.py`~~ | 361 | Same false-positive exclusion as above — same unrelated `classify_wealth_distribution` method. |

**RESERVED-LINE flags.**

1. **The `_BRACKET_BY_ROLE` 8→4 role-to-wealth-bracket fold** (`wealth_distribution.py:59-68`)
   is explicitly "RATIFIED (owner ruling 2026-07-16, ADR075)" per the module's own docstring
   (26-36), with named clarifications (CARCERAL_ENFORCER kept distinct from LABOR_ARISTOCRACY
   despite sharing a bracket; INTERNAL_PROLETARIAT/PERIPHERY_PROLETARIAT folded into one w4
   bracket). This is an owner-ratified class-theoretic content decision — described here, not
   re-litigated or proposed for change. (The market-scissors sibling inventory reaches the same
   conclusion independently for its own use of this identical table: "a settled non-ideological
   data mapping, not the National Question's B+C+I partition" — this inventory concurs but notes
   it is nonetheless a Director/owner-ratified MLM-TW class-theoretic mapping, not merely an
   arbitrary implementation detail.)
2. **The equilibrium wealth-share calibration targets and their empirical justification**
   (`tests/unit/config/test_wealth_distribution_invariants.py`, module docstring) frame the
   `equilibrium_w1..w4` bands as "laws of the capitalist mode of production" with an explicit
   theoretical claim ("the Fundamental Theorem in empirical form... within capitalism, no reform
   era ever pushed the bottom half of the population above ~4% of net personal wealth" — 118-130)
   and an explicit owner-ruled CONDITIONALITY constraint ("must NEVER be asserted against a live
   simulation trajectory, where a communist revolution breaking the distribution is the mechanic
   working as designed," 33-40). Described here as load-bearing context for anyone porting these
   calibration defines; not proposed for change.

Neither doctrine-tree content, National Question parameters (B+C+I), nor terminal-outcome
definitions are read or written anywhere in `wealth_distribution.py` or `formulas/
class_dynamics.py` — the two items above are the full extent of this system's Director-reserved
surface.

---

## Adjudication (2026-08-12)

Adjudicated against the current dev tree (`9324482f`). This is the most self-disciplined report
in the batch: its §6 preamble independently caught and resolved the ADR195/196-vs-dev-HEAD
staleness trap before relying on the enum lane, and its two hardest findings (Q6, the flat-key
bug) hold under adversarial check. The one thing it over-attributes is **D102's reach**. Two
corrections and four confirmations.

1. **CORRECTION — Step 5's role read is NOT D102-blocked; the landed per-subject `:field` enum
   path already serves it, and a shipped pack already proves it.** D102 defers only §2.10's
   `field-of` accessor on an enum-declared field. The very paragraph that mints the deferral says
   the opposite about the other accessor: "An `:enum-type`-declared field is read through a
   `:field` binding exactly as any other node field is (§2.5); this section does **not** extend
   §2.10's `field-of` accessor … to enum-declared fields"
   (`docs/reference/bsl-language.rst:2274-2284`). Step 5 is the ordinary per-subject rule form —
   `(binding role :field social-class/role)`, `(when (= role SocialRole/…))`,
   `(update-node self social-class/wealth-share …)` — and that is verbatim the shape
   `rust/crates/babylon-tick/content/rules/organization.bsl:26-29` ships today
   (`(binding kind :field organization/kind)` + `(when (and (= active 1) (= kind
   OrgKind/STATE_APPARATUS)))`), under a header comment that spells the distinction out: "read
   back through a `:field` binding — NOT `field-of`, which is refused" (`organization.bsl:4`).
   The row's *verdict* survives on its second leg — the projected value `shares[bracket]` is Q6
   graph-scope state with no read source — but the row is **singly** blocked, on Q6, not "doubly
   blocked."
2. **CORRECTION (same root) — the `_BRACKET_BY_ROLE` row's "once D102 is resolved" is not a real
   gate for the per-subject use.** A `defenum SocialRole (…)` plus a chain of `=` comparisons is
   expressible **today** wherever the role is read off the rule's own subject; D102 gates it only
   inside a fold body, where the ITERATED element's role must go through `field-of`. The
   classification table itself is portable now; only its use in `_bracket_resistances` is not.
3. **CONFIRMATION — Step 2's D102 blocker is real and correctly identified, and the mechanism is
   exactly as the report reasons.** `_bracket_resistances` (`wealth_distribution.py:172-180`)
   folds over ALL `social_class` nodes reading each ELEMENT's role. Because the rule's own
   subject would also be `social_class`, a `:field social-class/role` binding resolves against
   `self`, never against `it` — the implementation makes this unambiguous:
   `subject_type_of` requires every `:field` binding to share one namespace and errors otherwise
   ("a field is a field OF self's node type", `rust/crates/babylon-bsl/src/tick.rs:158-181`), and
   `bind_subject` seeds `self` and resolves each binding per subject (`tick.rs:218-250`). So the
   element read must be `(field-of it social-class/role)`, refused at LOAD citing D102 by name —
   `typecheck.rs:246-280`, wired at `rule_pipeline.rs:51,293-301`, vectored at
   `tick.rs:848,1304-1335`. The independent `E-TYPE-042` finding also holds
   (`bsl-language.rst:2547,2585`; `typecheck.rs:52,66`) — and, pointedly, the typechecker's own
   fixture uses `"wealth-share"` as its intensive-field example (`typecheck.rs:704`).
4. **CONFIRMATION — Q6 is real and doubly proven, beyond what the report claims.** `the` is
   `("the", "slice 2")` in `UNSERVED_EXPRESSION_HEADS` (`evaluator.rs:506`), exactly as cited.
   The stronger fact, confirmed here: `(domain :graph)` does not execute *at all* — zero
   `RuleDomain::Graph`/`loaded.domain` references in `babylon-bsl/src/tick.rs` or
   `babylon-tick/src/lib.rs`, and `subject_type_of` refuses a rule with no `:field` binding
   outright, with a message that states the limit as policy: "slice 1 runs rules over a
   population, not over the graph as a whole" (`tick.rs:170-172`). §5's architectural
   corroboration also checks out: `BabylonGraph.graph` is a live raw-dict property
   (`src/babylon/topology/graph.py:337-340`) that `GraphProtocol` never declares.
5. **CONFIRMATION — the flat-`class_consciousness` bug and the ZERO-downstream-reader claim, both
   exact.** The pre-registered exemption sits at `src/babylon/sentinels/vocabulary/registry.py:432-449`
   with the quoted reason verbatim and `tracking_task="#45"`; the flat read is
   `wealth_distribution.py:178`. And `wealth_share` has, across all of `src/babylon/`, exactly
   four non-formula hits: the `SocialClass` model declaration (`social_class.py:312`), two
   docstrings, and this system's own write (`wealth_distribution.py:274`) — no reader anywhere in
   the engine, projection, or intelligence layers. Both claims survive.
6. **CONFIRMATION — the `**2` flag, the coverage claims, and one addition worth carrying.**
   `frequency**2` is at `src/babylon/formulas/class_dynamics.py:228` verbatim, and the
   `(mul frequency frequency)` reformulation is right (there is no `pow` intrinsic —
   `DECLARABLE_INTRINSICS` is `exp`/`log`/`floor`). The `WealthDistributionSystem`/`C001.wealth_share`
   coverage claims are at `tools/regression_scenarios.py:388-394` and `:802-807`, in the same
   block as `MarketScissorsSystem`/`market_correction` (`:367-373`) — the live cross-system
   channel, exactly as stated. **Addition (non-material to the verdict, material to the eventual
   oracle):** `graph_content_hash` explicitly excludes graph metadata — "Graph *metadata*
   (`g.graph`: economy, event log, opposition states) is also excluded"
   (`tools/regression_test.py:934-941`) — so the byte-gate covers the per-node `wealth_share`
   projection but **not** the national `shares`/`velocities` vector itself. A conformance oracle
   built for this system must pin the vector by some route the byte-gate does not already supply.

**FINAL VERDICT: BLOCKED — sustained, on a narrower and better-attributed pair of gaps than the
report states. (i) Q6 stands and is stronger than claimed: the whole system's state is
graph-scope, `the` is Slice-2 unserved AND `(domain :graph)` does not execute at all on dev, so
there is no storage home and no once-per-tick firing shape either. (ii) D102 stands for
`_bracket_resistances`' element-wise role read inside a fold body — but NOT for Step 5's
per-subject projection nor for the `_BRACKET_BY_ROLE` table itself, both of which the landed
`:field` + `=` enum path serves today and `organization.bsl` already ships. The scalar
arithmetic remains D-record-ready, the `**2` reformulates without a new intrinsic, and the
frozen system's resistance input remains provably dead code regardless.**
