# SurvivalSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `SurvivalSystem` (165 lines, `src/babylon/engine/systems/survival.py`)
computes P(S|A)/P(S|R) for every non-territory node via two formula-registry calls
(`formulas/survival_calculus.py`) plus one local helper (`_calculate_solidarity_multiplier`).
The base sigmoid/ratio arithmetic is trivial arithmetic and mechanically portable once its
one hard dependency clears: the solidarity multiplier sums an **edge attribute**
(`solidarity_strength` on incoming `SOLIDARITY` edges) — genuinely BLOCKED, needs Slice 2 (the
dyadic edge lane), not served by the Slice-1 query-evaluation train that unblocked Territory.
Independent of that, the sigmoid's `math.exp()` call sits on a **second, separate** BSL gap:
`exp` is *declarable* (parses/typechecks/loads) but its evaluator dispatch does not exist on
dev today (`KernelIntrinsicHost` implements `floor` only — verified live,
`rust/crates/babylon-bsl/src/intrinsic_host.rs:59-70,269-277`) — a correction to this
inventory's own briefing, not a restatement of it. Layered on both of those is the ADR173
standing-law PORT-QUESTION: whether the logistic P(S|A) should even be transcribed as a
stipulated curve. The system also carries two verbatim defects (a comment/behavior mismatch
that lets it write onto non-`SocialClass` node types, and a module-level `GameDefines()` that
freezes `EPSILON` independent of the runtime `services.defines`), a graph-level dict-shaped
"policy delivery" read with no BSL representation (provably dormant on every canonical
scenario), and zero event emissions.

**Verdict: BLOCKED on Slice 2 (edge-attribute reads) for the solidarity multiplier, and
separately BLOCKED on `exp` evaluator dispatch for the sigmoid — the base ratio/sigmoid
arithmetic and every plain `:field` read are PORTABLE NOW/WITH D-RECORD, but the two
blockers gate the system's actual P(S|R) and P(S|A) outputs respectively, so the pack as a
whole is not portable today.**

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/survival.py` | 165 | **The target.** `SurvivalSystem`, one phase, one `step()`. Also hosts the module-level helper `_calculate_solidarity_multiplier` (lines 29-61). Imports: `engine.systems.policy.POLICY_DELIVERY_ATTR`, `kernel.tick_partition.TickPartition`, `models.enums.EdgeType`, `kernel.system_base.SystemBase`, `kernel.system_protocol.ContextType` (survival.py:17-26). No direct import of `formulas/` — the two formulas are pulled at runtime through `services.formulas.get(...)` (the hot-swap registry), not a static import. |
| `src/babylon/formulas/survival_calculus.py` | 111 | Coefficient source for the math: `calculate_acquiescence_probability` (21-43), `calculate_revolution_probability` (46-65) — both actually called by `SurvivalSystem.step()`. Also defines `calculate_crossover_threshold` (68-92) and `apply_loss_aversion` (95-110) — registered in the formula registry (`"crossover_threshold"`/`"loss_aversion"`) but **NOT called anywhere in `SurvivalSystem.step()` or by any other System's `step()`** (grep-confirmed across `src/babylon/engine/systems/*.py`); out of scope for this port, noted for completeness only. |
| `src/babylon/engine/formula_registry.py` | — | `FormulaRegistry.create_default()` registers `"acquiescence_probability"` → `calculate_acquiescence_probability` (line 106) and `"revolution_probability"` → `calculate_revolution_probability` (line 107). |
| `src/babylon/config/defines/survival.py` | 324 (whole module); `SurvivalDefines` lines 12-53 | `steepness_k` (18-22, default 10.0, `gt=0.0`, **no upper bound**), `default_subsistence` (23-28, `[0,1]`, default 0.3), `default_organization`/`default_repression`/`revolution_threshold`/`repression_base` (31-53) — the last two are **DEAD**: not read anywhere in `SurvivalSystem`'s actual math (see §2e). `BehavioralDefines.loss_aversion_lambda` (101-105) — only consumed by the unused `apply_loss_aversion`. |
| `src/babylon/config/defines/tunables.py` | — | `PrecisionDefines.epsilon` (39-44, `(0, 1e-3]`, default `1e-9`) — the division-by-zero guard `calculate_revolution_probability` uses, but see the module-level-freeze defect (§2, §4). |
| `src/babylon/config/defines/_assembler.py` | — | `GameDefines.DEFAULT_ORGANIZATION` (261-264 → `survival.default_organization`), `.DEFAULT_REPRESSION_FACED` (266-269 → `survival.default_repression`), `.DEFAULT_SUBSISTENCE` (271-274, **unused by survival.py**, which reads `services.defines.survival.default_subsistence` directly at line 105 instead), `.REPRESSION_BASE`/`.REVOLUTION_THRESHOLD` (251-259, both dead — see §2e). |
| `src/babylon/data/defines.yaml` | survival block 163-169; behavioral 192-193; precision 363-367 | Player-editable coefficient values. |
| `src/babylon/models/entities/social_class.py` | — | `SocialClass`/`EconomicComponent` field types read/written by `SurvivalSystem`: `wealth: Currency` (57, 308), `subsistence_threshold: Currency` (58, 351), `organization: Probability` (355), `repression_faced: Probability` (169, 359), `p_acquiescence`/`p_revolution: Probability` (160-161, 341-348), `active: bool` (380), `population: int` (406). |
| `src/babylon/models/entities/organization.py` | 116+ | `Organization` base entity — declares **none** of `wealth`/`population`/`organization`/`repression_faced`/`subsistence_threshold`/`active`/`p_acquiescence`/`p_revolution` (grep-confirmed zero hits). Load-bearing for the defect in §2/§3: `SurvivalSystem` writes `p_acquiescence`/`p_revolution` onto `ORGANIZATION` nodes too, which this model never declares. |
| `src/babylon/models/entities/relationship.py` | — | `solidarity_strength: Coefficient` (108-118, `[0,1]`, default 0.0) — the edge attribute `_calculate_solidarity_multiplier` reads. |
| `src/babylon/engine/actions/_mass_work.py` | — | `_MAX_SOLIDARITY_STRENGTH = 1.0` (59) — the writer-side cap on any *single* edge; the multiplier sums **multiple** such edges, so the sum itself is unbounded above (verified by `test_high_solidarity_multiple_edges`, §7). |
| `src/babylon/models/types.py` | 337 | `Probability` (50-58, `[0,1]`), `Currency` (104-120ish, `[0,∞)`), `Coefficient` (156-164, `[0,1]`) — all `Annotated[float, ...]` with `SnapToGrid` (1e-5 grid) applied only at Pydantic *instantiation*, never mid-tick. |
| `src/babylon/models/graph.py` | — | `GraphNode.node_type: str` (115) — plain string, not `NodeType`-typed; `GraphEdge` (147+) — `attributes: dict[str, Any]`, no type enforcement. |
| `src/babylon/models/enums/topology.py` | — | `NodeType` (SOCIAL_CLASS/TERRITORY/ORGANIZATION/INSTITUTION/INDUSTRY/SOVEREIGN/FACTION); `EdgeType.SOLIDARITY`. |
| `src/babylon/kernel/tick_partition.py` | — | `TickPartition.CONSEQUENCE` (18-28). |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase` — `survival.py` does **not** call `_write_clamped`, `_publish`, or `_wrap_graph`; it calls `graph.update_node`/`graph.query_nodes`/`graph.query_edges`/`graph.get_graph_attr` directly. |
| `src/babylon/kernel/graph_protocol.py` | 494 | `GraphProtocol.update_node` (88-98, plain merge semantics documented), `.query_nodes`/`.query_edges` (258-288), `.get_graph_attr`/`.set_graph_attr` (350-372). |
| `src/babylon/topology/graph.py` | — | `BabylonGraph.update_node` (660-670) — **plain dict merge, zero quantization/clamping mid-tick** (same finding as the Territory inventory). |
| `src/babylon/engine/systems/policy.py` | — | `POLICY_DELIVERY_ATTR = "policy_delivery"` (107); `PolicySystem.step` (144-220) writes the graph-level dict `{class_id: {"delivered": ..., ...}}` at 17.47, **one tick after** Survival reads it at 15.0 — Survival always sees last tick's write (I-ORD). |
| `src/babylon/engine/systems/economic.py` | — | `ImperialRentSystem` (class at line 26, `position = 9.0`, line 37) — a **separate, duplicate call site** of the same two registered formulas (`calculate_acquiescence`/`calculate_revolution`, lines ~596-604) used to gate a comprador subsidy; also the writer of `repression_faced` (640) and `wealth` (636) that `SurvivalSystem` reads same-tick. Not part of `SurvivalSystem` itself — noted for the cross-system channel map (§5). |
| `src/babylon/engine/systems/struggle.py` | — | `StruggleSystem` (`position = 16.0`, line 235) — the primary downstream reader of `p_acquiescence`/`p_revolution` (336-339). |
| `src/babylon/engine/systems/epistemic_horizon.py` | — | `EpistemicHorizonSystem` (`position = 22.0`, line 220) — reads `p_acquiescence` (95). |
| `src/babylon/engine/simulation_engine.py` | 611 | `_SYSTEM_CLASSES` tuple (328-364) confirms `SurvivalSystem` sits between `DoctrineSystem` (14.7) and `StruggleSystem` (16.0). |
| `src/babylon/sentinels/seam/registry.py` | — | `survival_p_acquiescence`/`survival_p_revolution` `SeamEntry` rows (1517-1550) — `INSPECTOR`-scope, `MUST_BE_LIVE`, write-site cited as `survival.py::SurvivalSystem.step (:143)` (a stale line number — the actual write is line 165 in the current 165-line file; the formula call is at 154-158/160-163). |

**Reference BSL surface read for adjudication** (all read/grepped, anchors verified this
session):
- `ai/decisions/ADR197_bsl_query_evaluation_slice1_handoff.yaml` — the Slice-1 handoff record; the authoritative "Slice 2 = the dyadic edge lane, SCOPED, NOT BUILT" statement (lines ~44-50).
- `docs/reference/bsl-language.rst` — §2.5 (bindings, `:field` is node-scoped only, chapter C1 rulings on edge-attribute reads needing `field-of`/§2.10, not `:field`), the `EdgeCondition`/`sum_strength` table row (~800-820) naming `(fold sum (edges EdgeType/SOLIDARITY) (field-of it solidarity/strength))` as the eventual SPEC target for `event_evaluator.py:174-175`'s `solidarity_strength` read (the same shape `_calculate_solidarity_multiplier` computes), the §2.3/chapter-C4 `<domain>` ruling (~703-730, D43 ~4823) that a rule's domain is **exactly one `NodeType`**, and item 10 of the Phase-1-review conformance-vector list (~4165) naming edge/hyperedge `field-of` as still-pending work.
- `rust/crates/babylon-bsl/src/evaluator.rs:1186` — "an `EdgeRef` referent is unreachable today (no expression form...)".
- `rust/crates/babylon-bsl/src/intrinsic_host.rs` (full file read) — `KernelIntrinsicHost` (57-70) dispatches `"floor"` only; the doc comment (1-20) states `{exp, log}` are "declarable... but their evaluation is Phase 2 work this host does not perform"; test `an_undeclared_name_fails_loud_exactly_like_the_empty_host` (269-277) asserts `KernelIntrinsicHost.call("exp", ...)` **fails**.
- `rust/crates/babylon-bsl/src/declarations.rs:110,1177` — `DECLARABLE_INTRINSICS = ["exp", "log", "floor"]` (declaration/typecheck level only — see the above for why declaration ≠ evaluation).
- `rust/crates/babylon-tick/content/rules/{vitality,metabolism,lifecycle}.bsl` — "no scalar min/max in the grammar" precedent (metabolism.bsl:393, vitality.bsl:55, lifecycle.bsl:161), nested-`if` clamp convention.

## 2. COMPUTATION CATALOG (execution order, `SurvivalSystem.step`, survival.py:84-165)

### Setup (survival.py:101-114)
- **(a)** Look up the two formulas from the hot-swap registry; read the steepness/subsistence
  coefficients; read the graph-level policy-delivery ledger (last tick's write, if any).
- **(b)** `calculate_acquiescence_probability = services.formulas.get("acquiescence_probability")` (102); `calculate_revolution_probability = services.formulas.get("revolution_probability")` (103); `survival_steepness = services.defines.survival.steepness_k` (104); `default_subsistence = services.defines.survival.default_subsistence` (105); `sw_delivery = graph.get_graph_attr(POLICY_DELIVERY_ATTR, None)` (114).
- **(c) Reads:** the formula registry; `services.defines.survival.{steepness_k,default_subsistence}`; graph-level attr `policy_delivery`.
- **(d) Writes:** none (local bindings only).
- **(e) Defines:** `survival.steepness_k` (10.0, `(0,∞)` — **unbounded above**), `survival.default_subsistence` (0.3, `[0,1]`) — defines.yaml:164-165.
- **(f) Events:** none.

### Per-node loop (survival.py:116-165) — for every node `graph.query_nodes()` returns
- **(a)** Skip territories; skip inactive entities; read the five material inputs (wealth,
  population, base organization, repression, subsistence); apply an optional social-wage
  subsistence offset; normalize wealth to per-capita; compute the solidarity-derived effective
  organization; compute P(S|A) and P(S|R); write both back onto the node.
- **(b)** Exact sequence:
  1. `if node.node_type == "territory": continue` (118) — **bare string literal**, not `NodeType.TERRITORY` (defect, transcribe verbatim — see §3).
  2. `if not attrs.get("active", True): continue` (124-125).
  3. `wealth = attrs.get("wealth", 0.0)` (127); `population = attrs.get("population", 1)` (128); `base_organization = attrs.get("organization", services.defines.DEFAULT_ORGANIZATION)` (129); `repression = attrs.get("repression_faced", services.defines.DEFAULT_REPRESSION_FACED)` (130); `subsistence = attrs.get("subsistence_threshold", default_subsistence)` (131).
  4. **Social-wage offset** (133-139): `if sw_delivery: row = sw_delivery.get(node.id); delivered = row.get("delivered")...; if isinstance(delivered, (int,float)) and delivered > 0.0 and population > 0: subsistence = max(0.0, subsistence - float(delivered) / population)`.
  5. `wealth_per_capita = wealth / population if population > 0 else 0.0` (143) — guarded ternary divide.
  6. `solidarity_multiplier = _calculate_solidarity_multiplier(graph, node.id)` (147) — see below.
  7. `effective_organization = min(1.0, base_organization * solidarity_multiplier)` (152).
  8. `p_acq = calculate_acquiescence_probability(wealth=wealth_per_capita, subsistence_threshold=subsistence, steepness_k=survival_steepness)` (154-158).
  9. `p_rev = calculate_revolution_probability(cohesion=effective_organization, repression=repression)` (160-163).
  10. `graph.update_node(node.id, p_acquiescence=p_acq, p_revolution=p_rev)` (165).
- **(c) Reads:** `<self>.wealth` (`Currency`), `<self>.population` (`int`), `<self>.organization` (`Probability`), `<self>.repression_faced` (`Probability`), `<self>.subsistence_threshold` (`Currency`), `<self>.active` (`bool`), `<self>.node_type` (`str`, bare-compared); graph-level `policy_delivery` dict; incoming `EdgeType.SOLIDARITY` edges' `solidarity_strength` (`Coefficient`, via the helper).
- **(d) Writes:** `<self>.p_acquiescence`, `<self>.p_revolution` (both `Probability`) — on **every** node `query_nodes()` yields that is not `node_type == "territory"` and is `active` — see §3's defect finding, this is broader than `SocialClass` alone.
- **(e) Defines:** `services.defines.DEFAULT_ORGANIZATION` (= `survival.default_organization`, 0.1, `[0,1]`), `services.defines.DEFAULT_REPRESSION_FACED` (= `survival.default_repression`, 0.5, `[0,1]`) — defines.yaml:166-167. **Dead defines in the same category, confirmed unread anywhere in the actual math:** `survival.revolution_threshold` (1.0, `(0,∞)`) and `survival.repression_base` (0.5, `[0,1]`) — defines.yaml:168-169; their `_assembler.py` properties `REVOLUTION_THRESHOLD`/`REPRESSION_BASE` (251-259) are likewise never referenced by any System (grep-confirmed; the only other `revolution_threshold` symbols in the tree belong to unrelated `carceral.revolution_threshold` in `control_ratio.py` and `territory.revolution_threshold` in `config/defines/territory.py` — different define namespaces entirely, a naming collision across categories, not a shared value).
- **(f) Events:** **none.** Grep-confirmed zero `EventType`/`.publish(`/`event_bus` references anywhere in `survival.py`.

### Helper: `_calculate_solidarity_multiplier` (survival.py:29-61)
- **(a)** Sum the `solidarity_strength` attribute of every `SOLIDARITY` edge whose target is
  this node; return `1.0 + that sum` (a multiplier, never below 1.0).
- **(b)** `for edge in graph.query_edges(edge_type=EdgeType.SOLIDARITY): if edge.target_id == node_id: solidarity_sum += edge.attributes.get("solidarity_strength", 0.0)` (54-58); `return 1.0 + solidarity_sum` (61).
- **(c) Reads:** ALL `SOLIDARITY`-typed edges in the graph (bulk `query_edges`, then a Python-side `target_id` filter — not a typed-incoming-neighbor query), each edge's `solidarity_strength` attribute.
- **(d) Writes:** none — pure function, return value only.
- **(e) Defines:** none.
- **(f) Events:** none.

### Formula bodies (`formulas/survival_calculus.py`)
- **`calculate_acquiescence_probability`** (21-43): `exponent = -steepness_k * (wealth - subsistence_threshold)` (41); `exponent = max(-500, min(500, exponent))` (42, overflow guard — hand-rolled two-sided clamp, not `_write_clamped`); `return 1.0 / (1.0 + math.exp(exponent))` (43) — **libm `exp` call**.
- **`calculate_revolution_probability`** (46-65): `if cohesion <= 0: return 0.0` (63-64); `return min(1.0, cohesion / (repression + EPSILON))` (65) — `EPSILON = _DEFINES.precision.epsilon` where `_DEFINES = GameDefines()` is a **module-level, import-time-frozen instance** (survival_calculus.py:16-17), independent of any `services.defines` a scenario or test might construct with an overridden `precision.epsilon` — see the defect note in §4.

## 3. TYPE INVENTORY

Runtime storage note (identical finding to the Territory inventory): `BabylonGraph.update_node`
(`topology/graph.py:660-670`) is a plain dict merge with **no type coercion or quantization**.
`SnapToGrid` (1e-5 grid) applies only at Pydantic model instantiation (scenario seed / a
`WorldState` round-trip), never mid-tick — every value below is raw Python `float`/`int` in
the tick loop.

| Attribute | Node type(s) actually reached | Python model type | Domain | Category |
|---|---|---|---|---|
| `wealth` | SOCIAL_CLASS declared; **ORGANIZATION/INSTITUTION/SOVEREIGN/FACTION/INDUSTRY also read via `attrs.get(..., 0.0)` default** | `Currency` (SocialClass only) | `[0.0, ∞)` | unbounded real, money-semantic |
| `population` | as above | `int` (SocialClass only; default `1` elsewhere) | `≥ 0` | integer |
| `organization` | as above | `Probability` (SocialClass only; default 0.1 elsewhere) | `[0,1]` | unit-interval |
| `repression_faced` | as above | `Probability` (SocialClass only; default 0.5 elsewhere) | `[0,1]` | unit-interval |
| `subsistence_threshold` | as above | `Currency` (SocialClass only; default 0.3 elsewhere) | `[0,∞)` | unbounded real, money-semantic |
| `active` | every non-territory node type (default `True` if absent) | `bool` (SocialClass only; defaulted True elsewhere) | `{T,F}` | boolean gate |
| `node_type` | every node | `str` (bare, not `NodeType`) | closed vocabulary, compared as a raw string literal `"territory"` | **enum-shaped string, compared without the enum** |
| `p_acquiescence` (write) | SOCIAL_CLASS declared; **also written onto ORGANIZATION/INSTITUTION/SOVEREIGN/FACTION/INDUSTRY nodes, none of which declare the field** | `Probability` | `[0,1]` (bounded by formula construction — sigmoid) | unit-interval |
| `p_revolution` (write) | same as above | `Probability` | `[0,1]` (bounded by formula construction — early-return + `min`) | unit-interval |
| `solidarity_strength` (edge attr, read) | `SOLIDARITY` edges only | `Coefficient` | `[0,1]` per-edge (writer-capped at 1.0, `_mass_work.py:59`); **the SUM across multiple incoming edges is unbounded above** | unit-interval-per-instance, unbounded aggregate |
| `policy_delivery` (graph-level attr, read) | N/A — graph scope, not a node/edge attribute | plain `dict[str, dict[str, Any]]`, unvalidated | keyed by `class_id`, each row `{"delivered": float\|None, ...}` | **graph-level keyed record store — no node/edge/hyperedge shape at all** |
| `survival.steepness_k` (define) | — | `float` | `(0.0, ∞)`, **no upper bound**, default 10.0 | unbounded real coefficient |
| `survival.default_subsistence`, `.default_organization`, `.default_repression` (defines) | — | `float` | `[0,1]` | unit-interval coefficients |
| `precision.epsilon` (define, module-frozen) | — | `float` | `(0, 1e-3]`, default `1e-9` | unbounded-below-in-practice engineering guard |

**Defect finding 1 — comment/behavior mismatch, broader write surface than documented.**
The docstring/inline comment claims `SurvivalSystem` processes "only social_class and untyped
nodes" (survival.py:117). The actual guard is `node.node_type == "territory": continue`
(118) plus the `active` check — nothing restricts the loop to `SOCIAL_CLASS`. **Verified live
on the canonical `org_probe` scenario** (`tools/regression_scenarios.py:128-193`, factory
`src/babylon/engine/scenarios/org_probe.py`, full file read): it seeds one `CivilSocietyOrg`
and one `StateApparatus` (both `_node_type=NodeType.ORGANIZATION` via
`WorldState.to_graph():750`, `G.add_node(org_id, _node_type=NodeType.ORGANIZATION,
**org.model_dump())`), neither of which sets `active` (Organization's Pydantic model has no
such field — confirmed by grep, zero hits in `models/entities/organization.py`), so
`attrs.get("active", True)` defaults `True` and both organizations run the full
computation every tick using the `attrs.get(key, default)` fallbacks (`wealth=0.0`,
`population=1`, `organization=0.1`, `repression_faced=0.5`, `subsistence_threshold=0.3`),
producing real, deterministic, non-degenerate `p_acquiescence`/`p_revolution` values written
onto ORGANIZATION nodes. Because `Organization` declares neither field,
`WorldState.from_graph()` cannot round-trip them back into `state.organizations` — the write
is real in-tick CPU/graph-mutation work but **silently dropped** at the next
`WorldState`↔`graph` boundary (confirmed: `tools/regression_test.py:924-943`'s
`graph_content_hash` hashes the `WorldState→graph` *projection*, not the live mid-tick graph
a `step()` builds and discards — so this particular write is not even byte-gate-visible
today). Transcribe verbatim, port-as-is (III.11): the defect is real, its effect is currently
inert on the canonical estate for a *second*, independent reason (round-trip loss), and it is
recorded here rather than silently narrowed to `SOCIAL_CLASS` in the port.

**Defect finding 2 — the enum-shaped `node_type` compare is a bare string, not `NodeType.*`.**
`node.node_type == "territory"` (118) — same pattern found in `struggle.py:191,316`
(not unique to Survival, so likely an estate-wide convention rather than a one-off mistake),
but it is the opposite direction of the vocabulary-sentinel's stamping rule (CLAUDE.md
Gotchas): this is a *read*-side bare compare, not a stamp. Transcribe verbatim.

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `float`), in execution order:

1. **Guarded divide (ternary):** `wealth / population if population > 0 else 0.0` (survival.py:143) — no epsilon, a full branch guard instead.
2. **Guarded subtract with lower clamp:** `max(0.0, subsistence - float(delivered) / population)` (survival.py:139) — a second guarded divide (`population > 0` checked in the enclosing `if`, line 136) plus a **lower-only** clamp via `max`.
3. **Multiply with upper-only clamp:** `min(1.0, base_organization * solidarity_multiplier)` (survival.py:152).
4. **Additive accumulation over a filtered edge set:** `solidarity_sum += edge.attributes.get("solidarity_strength", 0.0)` (survival.py:58) — order depends on `graph.query_edges()`'s iteration order (**UNVERIFIED whether that order is declared-deterministic**; `GraphProtocol.query_edges` (`graph_protocol.py:278-288`) documents only that it returns an iterator "for DuckDB compatibility (lazy evaluation)", not an ordering guarantee — search run: `rg -n "def query_edges" -A 20` on both `graph_protocol.py` and `topology/graph.py` found no explicit sort/order clause in either).
5. **Negate-subtract-multiply:** `exponent = -steepness_k * (wealth - subsistence_threshold)` (survival_calculus.py:41).
6. **Two-sided hand-rolled clamp:** `exponent = max(-500, min(500, exponent))` (survival_calculus.py:42) — the overflow guard for the next line's `math.exp`; `-500`/`500` are **bare integer literals** mixed into float arithmetic (Python auto-promotes; BSL's "no bare non-integer literal" rule is about literals like `1.0`, so these two ARE integer literals — a different, milder transcription question: do they enter as `Int` operands promoted to `Real` per §3.3, or must they be `500c`-suffixed reals? — **UNVERIFIED against the grammar's literal-suffix rules for this specific case; flagged, not resolved**).
7. **`exp` — LIBM TRANSCENDENTAL, NONDETERMINISM HAZARD:** `math.exp(exponent)` (survival_calculus.py:43). Two independent problems, not one: (i) cross-language/cross-platform bit-exact reproduction of `exp()` is not guaranteed by IEEE-754 the way `+ − × ÷` is (per this project's own behavioral-contract doctrine, a written tolerance-policy derivation is needed for any cross-implementation check); (ii) **on the current dev tree, `exp` is not evaluable at all** — see §6.
8. **Add-then-divide with sum-in-denominator, no floor of ratio:** `1.0 / (1.0 + math.exp(exponent))` (survival_calculus.py:43) — `1.0` is a **bare non-integer literal** (the same BSL grammar constraint the Territory inventory flagged; needs `1c`/promotion idiom).
9. **Divide with additive epsilon guard, upper-only clamp:** `min(1.0, cohesion / (repression + EPSILON))` (survival_calculus.py:65) — `EPSILON` is module-frozen (see defect below), not a live `services.defines` read.
10. **Early-return branch (no arithmetic):** `if cohesion <= 0: return 0.0` (survival_calculus.py:63-64).

**No `min`/`max` scalar clamp exists in the BSL grammar** — confirmed still current
(`content/rules/metabolism.bsl:393`, `vitality.bsl:55`, `lifecycle.bsl:161`: "§3.10's rider
slate declines a scalar min/max"). Every clamp above (items 2, 3, 6, 9) needs the nested-`if`
transcription the landed packs already use. Item 6's clamp is **two-sided** (both a floor and
a ceiling on the same value) — the landed-pack precedent (metabolism/vitality/Territory) is
all single-sided; a two-sided nested-`if` has no exact precedent yet in a landed pack
(port-time content-modeling note, not a blocker).

**Defect finding 3 — `EPSILON` is frozen at import time, independent of the runtime
`GameDefines`.** `survival_calculus.py:16-18`: `_DEFINES = GameDefines()` constructs a
**fresh, default-valued** `GameDefines` instance at module import, and
`EPSILON = _DEFINES.precision.epsilon` / `LOSS_AVERSION_COEFFICIENT =
_DEFINES.behavioral.loss_aversion_lambda` bake those two values in permanently. `GameDefines`'s
own docstring (`_assembler.py:99`) states its purpose is to "Override them per-scenario for
calibration" — but `calculate_revolution_probability`'s `EPSILON` never reads the
`services.defines` instance `SurvivalSystem.step()` is actually handed (survival.py never
even passes `services.defines.precision` into the formula call — the formula ignores its
caller's defines entirely for this one constant). Any scenario/test that constructs a
`GameDefines` with a non-default `precision.epsilon` would silently NOT affect this formula's
division-by-zero guard. Transcribe verbatim as a baked `:const` (port-as-is; this is the
frozen system's actual behavior, not a bug to fix in the port) and D-record the oddity.

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 15.0** (survival.py:78), `TickPartition.CONSEQUENCE`. Confirmed against
  `_SYSTEM_CLASSES` (`simulation_engine.py:328-364`): `... → DoctrineSystem (14.7) →
  SurvivalSystem (15.0) → StruggleSystem (16.0) → ConsciousnessSystem (17.0) → ...`.
- **Reads from same-tick prior systems (Material Base + Action + earlier Consequence
  positions):**
  - `wealth` — written earlier this tick by `VitalitySystem` (1.0, `vitality.py:122`),
    `ProductionSystem` (3.0, `production.py:181,192,198`), `ImperialRentSystem` (9.0,
    `economic.py:636` — the comprador-subsidy debit), `DecompositionSystem` (11.0,
    `decomposition.py:336`).
  - `population` — written earlier this tick by `VitalitySystem` (1.0, `vitality.py:132`),
    `TerritorySystem` (2.0, territory-side only — does not touch `SOCIAL_CLASS.population`),
    `DecompositionSystem` (11.0, `decomposition.py:336`).
  - `organization` — written earlier this tick **only** by `TerritorySystem`'s
    `_suppress_organization` (2.0, `territory.py:378`, PENAL_COLONY-gated, cross-node-type
    hard-set to `0.0` on `SOCIAL_CLASS` sources of `TENANCY` edges into a suppressed
    territory — matches the Territory inventory's own finding).
  - `repression_faced` — written earlier this tick by `ImperialRentSystem`'s subsidy branch
    (9.0, `economic.py:640`).
  - `active` — written earlier this tick by `DecompositionSystem` (11.0,
    `decomposition.py:336,339` — `active=True` on the newly-created Internal Proletariat,
    `active=False` on the decomposed Labor Aristocracy).
  - `subsistence_threshold` — **no System writes this attribute via `update_node` anywhere**
    (grep-confirmed zero hits across `src/babylon/engine/systems/*.py`); it is scenario-seed
    static state for the life of a run, modulated only by Survival's own same-tick,
    non-persisted, local `subsistence` variable (the social-wage offset, line 139, never
    written back to the graph).
  - `policy_delivery` (graph-level) — written by `PolicySystem` (17.47,
    `policy.py:172-218`) **one tick after** Survival reads it (15.0 < 17.47): the I-ORD
    one-tick lag documented in the survival.py comment itself (107-113) and in
    `policy.py:92-94`.
  - `solidarity_strength` (edge attr) — written by `SolidaritySystem` (8.0,
    `solidarity.py:155`, propagation), `DoctrineSystem` (14.7, `doctrine.py:138`, decay),
    `StruggleSystem` (16.0 — **after** Survival, so this write feeds only next tick),
    `CommunitySystem` (6.0, `community.py:574`, amplification), and the
    `_mass_work.py` action handlers (14.0, OODA-dispatched). All of these except
    `StruggleSystem` run before position 15.0 and are visible to Survival same-tick.
- **Writes consumed later this tick / downstream ticks:**
  - `p_acquiescence`, `p_revolution` — read by `StruggleSystem` (16.0, `struggle.py:338-339`,
    default `0.5`/`0.0` if absent — the rupture-condition input, "P(S|R) > P(S|A)"), and by
    `EpistemicHorizonSystem` (22.0, `epistemic_horizon.py:95`, fog-of-war shadow computation,
    `(1 - p_acquiescence) * class_consciousness * C_f`). Also exposed at `SeamScope.INSPECTOR`
    (`sentinels/seam/registry.py:1517-1550`) for the projection/observer layer (never read
    back into engine math from there).
- **Duplicate call site, not part of `SurvivalSystem` but touching the same formulas:**
  `ImperialRentSystem` (9.0, `economic.py:~596-613`) calls the **same two registered
  formulas** independently (its own local `p_acquiescence`/`p_revolution` variables) to decide
  whether a comprador client-state subsidy triggers. This is a separate computation with its
  own inputs (`target_organization`/`target_repression`, not `SurvivalSystem`'s node loop) —
  noted for completeness; out of scope for this port (it belongs to `ImperialRentSystem`'s own
  inventory).
- **Context/service usage with no BSL equivalent:**
  1. `graph.get_graph_attr(POLICY_DELIVERY_ATTR, None)` (survival.py:114) — a **graph-level
     keyed dict-of-records** read. BSL's only graph-scope carrier construct is `the`
     (bsl-language.rst chapter C3, D39-D40): "Graph-scope state is ordinary node state on a
     carrier `NodeType` whose ceiling is 1 — no new grammar, no second storage class" — i.e.
     exactly ONE node instance per carrier type, not a dict keyed by arbitrary `class_id`s
     with multiple sub-fields per row. **No existing BSL construct represents this shape** —
     distinct from (and, per the R9 chapter-C1 rulings, orthogonal to) the edge-attribute gap.
  2. `services.formulas.get(...)` (survival.py:102-103) — the hot-swap registry indirection
     itself has no BSL equivalent (BSL rules call declared intrinsics/arithmetic directly,
     there is no runtime formula-lookup-by-name concept) — mechanically irrelevant to the
     port (the port just inlines the formula body), noted for completeness.
- **DORMANCY on canonical scenarios.**
  1. **The solidarity-multiplier edge-attribute read is provably a no-op (multiplier ≡ 1.0)
     on every canonical `qa:regression` scenario.** `tools/regression_scenarios.py:2832-2840`
     (the `COVERAGE_GAPS_DATA` entry for `SolidaritySystem`, verified live): "every SOLIDARITY
     edge in all five scenarios has solidarity_strength=0.0 (imperial_circuit's scenario-seed
     default; two_node has no SOLIDARITY edge at all)". The same root condition is repeated
     as a declared at-rest reason across a dozen other coverage-gap rows in the same file
     (e.g. lines 691-692, 710-711, 878-879, "SOLIDARITY (potential internationalism,
     solidarity_strength=0.0 in every canonical scenario)"). This means
     `_calculate_solidarity_multiplier` returns exactly `1.0` on every seeded SOLIDARITY edge
     today, and `effective_organization` collapses to `base_organization` unmodified —
     the BLOCKED edge-attribute read is, today, provably a constant identity operation on the
     canonical estate (the same "provably uniform" shape as Territory's
     `displacement_mode` and Metabolism's D-2).
  2. **The social-wage offset (survival.py:133-139) is provably dead on every canonical
     scenario.** `PolicySystem`'s own docstring (`policy.py:39-42`) states: "None of the six
     qa:regression scenarios ever carries either register [agenda or fiscal], so all six are
     byte-unchanged with the system live" — `POLICY_DELIVERY_ATTR` is therefore always absent,
     `sw_delivery` is always `None`, and the `if sw_delivery:` branch never executes on the
     canonical estate. **Search run for a counter-example:** `rg -n
     "POLICY_DELIVERY_ATTR|POLICY_AGENDA_ATTR|enqueue_agenda_item|policy_agenda"
     tools/regression_scenarios.py` → one hit, an unrelated `policy_agenda_rate` coefficient
     override (line 83), never a seeded agenda item. Confirmed dormant.
  3. **The cross-node-type write (§3 defect finding 1) is LIVE, not dormant** — the
     `org_probe` scenario (part of the canonical `tools/regression_scenarios.py` registry,
     line 128) exercises it every tick, as detailed in §3.
  4. **The `active`/`repression_faced` inputs ARE live** on every canonical scenario that
     seeds `SOCIAL_CLASS` nodes (all of them) — this is the one part of the system that is
     genuinely exercised end-to-end by the byte-gate today (modulo the round-trip-loss caveat
     for non-`SocialClass` node types noted in §3).

## 6. BLOCKER ASSESSMENT

Adjudicated against the CURRENT BSL surface as stated in this task's briefing, corrected
where this session's own anchor-reading diverges from it (see the executive summary and the
`exp`-evaluation finding, which is a genuine correction, not a restatement).

| Computation | Verdict | Detail |
|---|---|---|
| `node.node_type == "territory"` skip (survival.py:118) | **PORTABLE NOW** | A plain `:field`/`self` comparison against a `NodeType` enum member (once the rule's `<domain>` is declared per node type — see the ORGANIZATION-write row below). |
| `active` skip, material-input reads (`wealth`/`population`/`repression_faced`/`subsistence_threshold`/`organization`, survival.py:124-131) | **PORTABLE NOW** | Plain `:field self` reads with `:default` fallbacks — the same shape every landed pack already uses. `population` is `int`, others are plain `Real`-lane fields; no query/fold/edge involvement. |
| Social-wage subsistence offset (survival.py:133-139) | **PORTABLE WITH D-RECORD** | The read (`graph.get_graph_attr(POLICY_DELIVERY_ATTR)`) has **no BSL representation** — not a node field, not an edge field, not the single-instance `the` carrier (§5). Provably dormant on every canonical scenario (§5) — declare the effective relief `:const 0` under the same "provably uniform" reasoning Territory used for `displacement_mode`, and D-record that the underlying graph-level dict-ledger construct remains unrepresented for whenever `PolicySystem` itself is ported. |
| Per-capita wealth divide (survival.py:143) | **PORTABLE NOW** | A guarded ternary divide — trivial `<arith>` plus a comparison; matches the landed-pack "guard, then divide" idiom. |
| `_calculate_solidarity_multiplier` (survival.py:29-61, called at 147) | **BLOCKED — Slice 2 (the dyadic edge lane)** | Needs `field-of` over an `EdgeRef` to read `solidarity_strength`, per `ai/decisions/ADR197...yaml`'s own words: "slice 2 (the dyadic edge lane — edges, edge-between, field-of over an EdgeRef, the) needs a new Value::EdgeRef/EdgeKey type and one read-only edge-attribute lookup" — SCOPED, NOT BUILT. `evaluator.rs:1186` confirms no expression form reaches an `EdgeRef` today. **Provably a constant (`1.0`) on every canonical scenario** (§5) — the port can declare `:const 1.0` for the multiplier under the same "provably uniform" reasoning used above, UNTIL a scenario ever seeds `solidarity_strength > 0`, at which point the port genuinely needs Slice 2. |
| `effective_organization = min(1.0, base_organization * solidarity_multiplier)` (survival.py:152) | **PORTABLE WITH D-RECORD** | The multiply and the upper-only clamp are trivial `<arith>` + nested-`if`; blocked only transitively through the solidarity multiplier above — same disposition. |
| `calculate_acquiescence_probability`'s sigmoid (survival_calculus.py:41-43) | **BLOCKED — `exp` evaluator dispatch, AND a PORT-QUESTION under ADR173** | Two independent findings, stacked: (1) **mechanical block**: `exp` is declarable (`DECLARABLE_INTRINSICS`, `declarations.rs:110`) but `KernelIntrinsicHost` — the only non-empty, non-test `IntrinsicHost` wired into production (`intrinsic_host.rs:57`) — implements `"floor"` only; a call to `"exp"` fails loud (`intrinsic_host.rs:62-68`, proven by the test at 269-277). The doc comment states plainly this is "Phase 2 work this host does not perform", gated on "pinned soft-float libm + golden vectors" (ADR176 r21). This is BLOCKED regardless of any ideological ruling. (2) **standing-law port-question, independent of (1)**: per ADR173/ADR172 ruling 5, the frozen logistic P(S|A) is the reference implementation's form, not the going-forward law — the Rust/BSL P(S|A) should EMERGE as the measure of class members whose wealth clears subsistence, not be stipulated as a sigmoid. Whether the port transcribes the sigmoid as-is (once `exp` evaluation lands) or reformulates per ADR173 is a Director-gate question — **RESERVED-LINE**, describing both readings without proposing between them. |
| The `[-500,500]` overflow-guard clamp (survival_calculus.py:42) | **PORTABLE WITH D-RECORD** | Nested-`if`, two-sided (no exact landed-pack precedent for a two-sided clamp yet — content-modeling note, not a blocker) — moot unless/until the `exp` call itself becomes portable. |
| `calculate_revolution_probability` (survival_calculus.py:63-65) | **BLOCKED, transitively** | The division/early-return/clamp shape is itself trivial `<arith>` (`PORTABLE NOW` in isolation), but its actual input `effective_organization` is blocked by the solidarity-multiplier row above until Slice 2 lands (or the `:const 1.0` D-record substitutes for it). `EPSILON` itself: **PORTABLE WITH D-RECORD** as a baked `:const` (transcribing the module-freeze defect verbatim, §4). |
| The cross-node-type write onto ORGANIZATION/INSTITUTION/SOVEREIGN/FACTION/INDUSTRY nodes (§3 defect finding 1) | **BLOCKED — no single-domain rule shape; NOT-A-PACK as currently coded** | BSL's `<domain>` construct is **exactly one `NodeType` member** (bsl-language.rst chapter C4, D43: `"|U| ≠ 1" is E-LOAD-004`); a rule cannot range over "every node type except territory". Faithfully porting the frozen system's actual behavior (port-as-is) would require either (a) one duplicate rule per reachable `NodeType`, hitting the open D-row on whether two rules at one anchor position share pre-state (Q14/D116, still open per this briefing) if more than one such rule needs the same anchor, or (b) a declared, named deviation narrowing scope to `SOCIAL_CLASS` only — which diverges from the frozen system's verified live behavior on `org_probe`. Neither option is free; this is a genuine, previously-unnamed content-modeling gap, named here for the eventual pack's D-record rather than resolved. |
| `p_acquiescence`/`p_revolution` write (survival.py:165) | **PORTABLE NOW** (mechanically) | `update-node` against `self` with two computed `Probability` fields — trivial once its inputs are portable; the write verb itself has no gap. |

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_survival.py` | 254 | **Primary system-level conformance oracle.** `TestPopulationNormalization` class: per-capita wealth normalization, backward-compat at population=1, inactive-entity skip, zero-population safety (no `ZeroDivisionError`), equal-per-capita-equal-P(S|A) invariant. Exercises `SurvivalSystem.step()` end-to-end against a hand-built `BabylonGraph`, not a canonical scenario. |
| `tests/unit/engine/systems/test_survival_helpers.py` | 263 | **Direct conformance oracle for `_calculate_solidarity_multiplier`** — the exact function this inventory's central blocker concerns. Eleven cases: no edges (→1.0), non-SOLIDARITY edges ignored, single/multiple edge accumulation, missing-attribute default, string-vs-enum edge-type equivalence, mixed edge types, outgoing-edges-not-counted (direction matters), zero-strength edge, high-solidarity compounding (>2.0), `edge_type=None` ignored. Every case uses `pytest.approx(..., abs=0.001)` — a ready-made numeric-tolerance policy for the eventual `.bscn` fixture. |
| `tests/unit/formulas/test_survival_calculus.py` | 438 | Example-based tests on the two formula bodies directly (sigmoid bounds, crosses-0.5-at-threshold, monotonicity, revolution-probability ratio behavior, crossover threshold, loss aversion) — including `calculate_crossover_threshold`/`apply_loss_aversion`, which `SurvivalSystem` itself never calls (§1). Good conformance-oracle candidate for the **formula layer**, not the system's graph-integration behavior. |
| `tests/unit/formulas/test_survival_calculus_properties.py` | 379 | **Hypothesis property-based tests** — `TestAcquiescenceProbabilityProperties`: P(S|A) always in `[0,1]` across the input space, exactly `0.5` at threshold, monotonic in wealth; analogous properties for P(S|R), crossover threshold, and loss aversion. This is the strongest behavioral-contract candidate in the estate for this system (per the project's own tests-as-behavioral-contracts doctrine) — these properties should re-verify independent of implementation language. |
| **Gap: no `tests/unit/engine/laws/test_law_survival_system.py`.** | — | Seventeen other Systems each have a dedicated property-based law-test file in `tests/unit/engine/laws/` (`test_law_territory_system.py`, `test_law_metabolism.py`, `test_law_solidarity.py`, etc. — confirmed via `ls`); **`SurvivalSystem` has none.** The closest analogues are the formula-level Hypothesis properties above, which do not exercise the system's graph-integration invariants (e.g. "P(S|A)/P(S|R) are always written for every active non-territory node", "the cross-node-type write never crashes on a node missing all five inputs") as a law. Flagged as a coverage gap for the eventual pack's conformance-oracle design, not fixed here. |
| `src/babylon/sentinels/seam/registry.py:1517-1550` | — | `survival_p_acquiescence`/`survival_p_revolution` `SeamEntry` rows — documents the INSPECTOR-scope contract (`MUST_BE_LIVE`, always present as a `SocialClass` `Probability` field) but is a projection/observer contract, not a tick-behavior conformance test. |

**qa:regression byte-gate coverage.** Per `tools/regression_test.py:924-943`
(`graph_content_hash`), every node/edge attribute of the `WorldState→graph` *projection* is
hashed — so `p_acquiescence`/`p_revolution` on `SOCIAL_CLASS` nodes are byte-gate-visible on
every canonical scenario that seeds active social classes (all of them). The
**cross-node-type write onto ORGANIZATION nodes is NOT byte-gate-visible** (§3: dropped at the
`from_graph()` round-trip boundary before hashing, since `Organization` declares no such
field) — a live defect with zero regression-test surface today. The solidarity-multiplier
path and the social-wage offset are both dormant on the canonical estate (§5) — any port
conformance fixture exercising either needs a hand-built `.bscn`, not the canonical
scenarios, matching the Territory inventory's own conclusion for its structurally-dormant
paths.

---

## Adjudication (2026-08-12)

Adjudicated against the **current dev tree** (`9324482f`). The `exp`-dispatch finding — this
inventory's own correction to its briefing — is right, was independently re-verified line by
line, and is the most useful single fact in the report. The two defect findings are real. The
verdict **BLOCKED stands and is hardened**: the escape hatch the report proposes for its own
primary blocker does not exist, because the dormancy premise it rests on is stale.

1. **CORRECTION — the solidarity multiplier is NOT provably `1.0` on the canonical estate, and
   the proposed `:const 1.0` D-record is invalid.** §5's dormancy item 1 and §6's
   `_calculate_solidarity_multiplier` row both rest on the `COVERAGE_GAPS_DATA` row for
   `SolidaritySystem` — "every SOLIDARITY edge in all five scenarios has
   `solidarity_strength=0.0`" (`tools/regression_scenarios.py:2833-2836`) — and conclude "the port
   can declare `:const 1.0` for the multiplier under the same 'provably uniform' reasoning."
   That row is stale by seven scenarios. **Two canonical scenarios seed a non-zero
   `solidarity_strength` today**: `create_debs_scenario` at
   `src/babylon/engine/scenarios/electoral_goldens.py:474` (`_solidarity(_WORKER, "C005", 0.4)`)
   and `create_bernie_valve_scenario` at `:534` (`_solidarity(_WAYNE_WORKER, "C006", 0.4)`), both
   through the `_solidarity` helper at `:156-163` which stamps
   `edge_type=EdgeType.SOLIDARITY, solidarity_strength=strength`. Both scenarios are registered in
   `SCENARIOS` (`tools/regression_scenarios.py:106,115`), both have committed
   checkpoint + dense baselines, `PENDING_CEREMONY` is empty (`:143`), and `qa:regression compare`
   iterates the whole twelve-key registry (`tools/regression_test.py:1424`). Since
   `_calculate_solidarity_multiplier` filters `edge.target_id == node_id` (`survival.py:55-58`),
   nodes `C005` and `C006` receive a multiplier of exactly `1.4`, not `1.0`, and
   `effective_organization = min(1.0, base_organization * 1.4)` (`survival.py:152`) diverges from
   `base_organization` on the byte gate. **Consequence:** Slice 2 is a LIVE blocker with a live
   byte-gated oracle, not a latent one; the `:const 1.0` deviation would produce a
   demonstrably wrong `p_revolution` on two canonical scenarios; and the sentence "the BLOCKED
   edge-attribute read is, today, provably a constant identity operation on the canonical estate"
   is withdrawn.

2. **CORRECTION — every "six"/"five" scenario count quoted in §5 is stale by six.** §5's dormancy
   item 2 quotes `policy.py:39-42` ("None of the six qa:regression scenarios ever carries either
   register"); item 1 quotes "all five scenarios." The registry has **twelve** keys —
   `imperial_circuit`, `two_node`, `starvation`, `glut`, `fascist_bifurcation`, `single_county`,
   `mitterrand`, `syriza`, `weimar`, `debs`, `bernie_valve`, `org_probe`
   (`tools/regression_scenarios.py:38-128`). The social-wage-offset conclusion may still survive
   (I found no scenario factory seeding a policy-agenda item, and `PolicySystem` @17.47 writes
   `POLICY_DELIVERY_ATTR` after Survival @15.0 reads it either way), but it is **asserted from a
   stale source rather than derived**, and must be re-derived over twelve — particularly against
   the five electoral goldens, whose entire subject is the policy/electoral machine.

3. **CONFIRMATION — `exp` is declarable but not dispatchable, and this is a real second blocker
   independent of Slice 2.** Verified by full read: `DECLARABLE_INTRINSICS: [&str; 3] = ["exp",
   "log", "floor"]` (`rust/crates/babylon-bsl/src/declarations.rs:110`), while
   `KernelIntrinsicHost::call` matches `"floor"` alone and returns
   `"no intrinsic registered: {other} (only 'floor' — ADR188 Row 2 — is implemented today; the
   {exp, log} transcendental cap and round-half-even remain Phase 2 work)"`
   (`intrinsic_host.rs:59-69`), with the module doc stating the same at `:50-56` and the test
   `an_undeclared_name_fails_loud_exactly_like_the_empty_host` (`:269-277`) asserting
   `call("exp", …)` fails. `KernelIntrinsicHost` is the production host — its own doc calls it
   "this crate's first non-empty, non-test-only `IntrinsicHost` — wired into
   `babylon-tick::run_once_into`" (`:17-20`). The report's correction to its own briefing is
   exact.

4. **CONFIRMATION — defect finding 1 (the broader-than-documented write surface) is real, and
   the round-trip-loss analysis is right.** `survival.py:117-119` reads verbatim: comment "Skip
   territory nodes (only process social_class and untyped nodes)", guard
   `if node.node_type == "territory": continue`. Nothing narrows the loop to `SOCIAL_CLASS`, and
   `Organization` declares neither `active` nor `p_acquiescence`/`p_revolution`, so `org_probe`'s
   two ORGANIZATION nodes run the full computation with the `attrs.get(key, default)` fallbacks
   and the writes are dropped at the `from_graph()` boundary before `graph_content_hash` sees
   them (`tools/regression_test.py:936-943` hashes the `WorldState→graph` **projection**, and its
   own docstring says the harness "never holds the *live* graph a tick mutates"). A live defect
   with zero regression surface, correctly transcribe-verbatim.

5. **CONFIRMATION — defect finding 3 (the import-time-frozen `EPSILON`) and defect finding 2 (the
   bare `"territory"` compare) both hold**, and the `:const`-transcription disposition for the
   former is the right port-as-is call.

6. **CONFIRMATION — the single-`NodeType` domain rule.** `E-LOAD-004` is real and is exactly the
   undeterminable/ambiguous-domain code (`rust/crates/babylon-bsl/src/domain.rs:15,49,83,96`), so
   §6's "a rule cannot range over 'every node type except territory'" row is sound, as is its
   naming of the open Q14/D116 two-rules-at-one-anchor pre-state row as the cost of option (a).

7. **CONFIRMATION — tick position.** `position: ClassVar[float] = 15.0` (`survival.py:78`),
   between `DoctrineSystem` (14.7, `doctrine.py:626`) and `StruggleSystem` (16.0,
   `struggle.py:235`) in `_SYSTEM_CLASSES` (`simulation_engine.py:328-364`). The downstream
   readerships (`struggle.py:336-339`, `epistemic_horizon.py:95`) re-grep clean, and the
   `repression_faced` producers are correctly two: `economic.py:640` (ImperialRent @9.0) and
   OODA's `action_effects.py:306,346,473` @14.0.

8. **CONFIRMATION — the RESERVED-LINE handling is correct.** The ADR172 ruling-5 / ADR173
   port-question on whether P(S|A) may be transcribed as a stipulated logistic at all is flagged
   as Director-gated and both readings are stated without proposing between them, exactly per
   mandate. Reinforcing anchor the report does not cite: `sigmoid` is the sole member of
   `PROHIBITED_INTRINSIC_NAMES` (`declarations.rs:116`), whose own comment says it "would hand
   content the exact mechanism ADR172 ruling 5 forbids, pre-packaged and named" — so the
   mechanical half of that gate is already enforced at load, and only the "re-derive as a
   measure" question is open.

**FINAL VERDICT: BLOCKED — sustained and hardened.** Slice 2 (edge-attribute reads) gates the
solidarity multiplier and a separate, independently-verified `exp` evaluator-dispatch gap gates
the sigmoid; plain `:field` arithmetic is PORTABLE NOW/WITH D-RECORD; neither of Survival's two
written outputs computes correctly without one of those two lanes. **What changes:** the
"provably uniform, declare `:const 1.0`" escape hatch for the solidarity multiplier is withdrawn
— `debs` and `bernie_valve` seed `solidarity_strength=0.4`
(`electoral_goldens.py:474,534`), so the Slice-2 blocker is live on the byte gate and a port that
constants it away would be measurably wrong. That also means the pack has a **real conformance
oracle waiting** on two canonical scenarios the moment Slice 2 lands, which is a materially
better position than the report concludes.

**INADEQUATE-COVERAGE NOTE (narrow).** A re-read must (i) re-derive both §5 dormancy claims over
the twelve-key `SCENARIOS` registry rather than quoting the stale "six"/"five" counts, naming per
scenario whether `solidarity_strength > 0` and whether a policy-delivery register is ever
populated; (ii) resolve the `query_edges` iteration-order question §4 item 4 leaves UNVERIFIED —
it is load-bearing for FP-summation determinism now that a non-zero multiplier is known to be
live on two scenarios (`src/babylon/topology/adapters/query_mixin.py` is the concrete
implementation to read, and the sibling Struggle inventory already reports it as unsorted
rustworkx insertion order at `query_mixin.py:50`).
