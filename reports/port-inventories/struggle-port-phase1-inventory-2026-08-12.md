# StruggleSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `StruggleSystem` (748 lines, `src/babylon/engine/systems/struggle.py`, position
16.0, `CONSEQUENCE` phase) is the Agency Layer: a stochastic Spark→Uprising pipeline (repression rolls
an `EXCESSIVE_FORCE` Bernoulli draw from a shared Python `random.Random` stream, and a hit plus high
agitation triggers wealth destruction + `SOLIDARITY`-edge strength increments + class-consciousness
gain), a deterministic George Jackson power-vacuum bifurcation (comprador insolvency routes to
Revolutionary Offensive or Fascist Revanchism by periphery `organization × class_consciousness`), a
deterministic peripheral revolt (severs outgoing `EXPLOITATION` edges when P(S|R) > P(S|A)), and a
deterministic spontaneous-riot gate for `LUMPENPROLETARIAT` (self-declared dormant on every canonical
scenario). It carries **two categorically new blockers** beyond anything Territory's inventory
surfaced: (1) the spark roll needs an RNG draw inside rule evaluation, and BSL's grammar **explicitly
prohibits a randomness primitive** — RNG draws are ruled kernel intrinsics, permanently outside content,
not merely an unbuilt lane; (2) every `SOLIDARITY`-edge strength increment (used by 7 systems, not just
this one) needs `update-edge`, which **is grammar-recognized and hard-refused** because `GraphSubstrate`
has no per-edge attribute storage at all — a declared substrate gap, not a query-lane gap. The
deterministic branches (spontaneous riot, the bifurcation's core arithmetic, peripheral revolt's edge
severing) are separately portable now or with D-records once role is content-modeled as a BSL `enum`
(a real, already-landed precedent exists: `organization/kind enum OrgKind`). Zero libm calls in this
system's own code, but it reads `p_acquiescence`/`p_revolution` written by `SurvivalSystem` in the SAME
tick via a stipulated `math.exp` sigmoid — a live ADR172/173 port-question, and independently blocked on
its own axis (`exp` is declarable/typechecks but has no evaluator dispatch — `KernelIntrinsicHost`
implements `floor` only, verified live). Eight distinct `EventType` emissions, all WS1 ledger rows.

**Verdict: BLOCKED (RNG-as-BSL-content-primitive, categorically prohibited; and the `update-edge`
substrate-storage gap) — the deterministic branches are PORTABLE NOW / WITH D-RECORD, but the system's
title mechanic (Spark → Uprising, including every `SOLIDARITY`-edge gain) cannot land as a pure BSL
content pack until both gaps close, and neither is "coming soon" the way Slice 1's query lane was.**

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/struggle.py` | 748 | **The target.** `StruggleSystem` (class at 213), one `step()` (278-457) plus four helper methods called at the end of `step()` — `_check_power_vacuum` (523-587, plus `_apply_revolutionary_offensive` 589-624 and `_apply_fascist_revanchism` 626-675), `_check_peripheral_revolt` (677-748), `_check_spontaneous_riot` (459-521) — and two more: `_legitimation_backfire_multiplier` (242-264), `_mean_legitimation` (266-276). Module-level helpers: `_get_agitation_from_node` (58-77), `_update_class_consciousness` (80-109), `_update_national_identity` (112-141), `_update_agitation` (144-173), `_find_entity_by_role` (176-210), `_STRUGGLING_ROLES` frozenset (50-55). |
| `src/babylon/formulas/reactionary.py` | 152 | `calculate_spontaneous_riot_risk` (94-117) — called by `_check_spontaneous_riot`. Also declares `calculate_entitlement_effective` (120-…) and (per grep) a `math.exp`-based defection-probability sigmoid at line 91 — **neither is called by struggle.py** (confirmed by reading every call site in struggle.py); noted for completeness only, out of scope. |
| `src/babylon/kernel/node_access.py` | 37 | `class_consciousness_from_node` (15-37) — called by `step()` (404, 414, 561) and `_check_power_vacuum`. Consolidates three formerly-duplicated copies (Solidarity/Struggle/ImperialRent, per its own docstring). |
| `src/babylon/kernel/system_base.py` | 202 | `resolve_rng` (35-55) — the stochastic-roll RNG source; `SystemBase` — struggle.py does **not** call `_write_clamped` anywhere (all clamps are hand-rolled, §4). |
| `src/babylon/kernel/tick_partition.py` | 30 | `TickPartition.CONSEQUENCE` (28). |
| `src/babylon/kernel/event_bus.py` | 288 | `Event` (33), `EventBus.publish` (134) — struggle.py's sole write path to the event system, 8 call sites. |
| `src/babylon/domain/bifurcation/legitimation.py` | 86 | `compute_legitimation_amplifier` (36-86) — called by `_legitimation_backfire_multiplier` when a government is seated. Population-weighted mean legitimation → linear affine amplifier, no libm. |
| `src/babylon/kernel/graph_protocol.py` | 494 | `query_nodes`/`query_edges` (258-288), `update_node` (88-98), `update_edge` (152-171), `remove_edge` (172-…), `get_node` (77-87), `get_graph_attr` (350-…). |
| `src/babylon/topology/adapters/query_mixin.py` | 147 | Concrete `query_nodes`/`query_edges` (34-116) — **unsorted, rustworkx insertion-order iteration** (`for node_id in self._graph.nodes`, line 50); load-bearing for the RNG-stream-order finding in §4/§5. |
| `src/babylon/topology/graph.py` | — | `BabylonGraph.update_node` (660-670, plain dict merge, no quantization), `.update_edge` (690), `.remove_edge` (262). |
| `src/babylon/models/entities/social_class.py` | 522 | `SocialClass` — field types/domains for every attribute this system reads/writes (§3); `IdeologicalProfile` (61-152, the nested `ideology` sub-model). |
| `src/babylon/models/enums/social.py` | 211 | `SocialRole` (12-…, 8 members) — struggle.py touches 4: `PERIPHERY_PROLETARIAT`, `LUMPENPROLETARIAT`, `COMPRADOR_BOURGEOISIE`, `LABOR_ARISTOCRACY`. |
| `src/babylon/models/enums/topology.py` | 253 | `NodeType.TERRITORY`/`.SOCIAL_CLASS`; `EdgeType.SOLIDARITY`/`.EXPLOITATION`. |
| `src/babylon/models/enums/events.py` | 234 | The 8 `EventType` members struggle.py emits (§2f). |
| `src/babylon/config/defines/survival.py` | 323 | `StruggleDefines` (175-247) — every `struggle.*` coefficient (§2e). |
| `src/babylon/config/defines/consciousness.py` | 582 | `repression_backfire` (103-108), `legitimation_amplifier_scale` (509-517). |
| `src/babylon/config/defines/politics.py` | 574 | `legitimacy_backfire_threshold` (434-444). |
| `src/babylon/config/defines/reactionary.py` | 160 | `spontaneous_riot_threshold` (94-99). |
| `src/babylon/config/defines/_assembler.py` | — | `GameDefines.DEFAULT_REPRESSION_FACED` (267-269 → `survival.default_repression`). |
| `src/babylon/data/defines.yaml` | struggle block 279-288; other rows below | Player-editable coefficient values. |
| `src/babylon/models/events/struggle_payloads.py` | 116 | Frozen Pydantic mirrors of 5 of the 8 event payloads (`PowerVacuumEvent`, `RevolutionaryOffensiveEvent`, `FascistRevanchismEvent`, `SpontaneousRiotEvent`, `PeripheralRevoltEvent`) — history note in its own docstring: these five "were emitted onto the bus but silently dropped by `_convert_bus_event_to_pydantic`" until a Program-17 wave widened the conversion whitelist. Python-side plumbing history, not a BSL-port fact. |
| `src/babylon/engine/simulation_engine.py` | 611 | `_SYSTEM_CLASSES` tuple (328-364) confirms position: `... SurvivalSystem(15.0) → StruggleSystem(16.0) → ConsciousnessSystem(17.0) → FascistFactionSystem(17.4) → AllegianceSystem(17.42) → ElectoralSystem(17.45) → ...`. |

**Not exercised by struggle.py:** no `domain/economics/*` module; `formulas/` contributes exactly one
call (`calculate_spontaneous_riot_risk`). No `services.formulas.get(...)` hot-swap-registry lookups
anywhere (unlike `SurvivalSystem`, which pulls its sigmoid/ratio formulas through the registry) —
struggle.py's own arithmetic is 100% inline.

**Current BSL surface read for adjudication** (all read/grepped, anchors verified this session):
- `docs/reference/bsl-language.rst` §2.8 "Prohibited" clause (1620-1628) — the explicit no-randomness
  ruling; §2.6 `neighbors` grammar and directionality (946, 998, 1079-1108); §2.6 R9 chapter C3 "graph-
  scope state is ordinary node state on a declared carrier node type" ruling (2650-2679) plus its
  rejected-alternative note (2674-2680); §2.10 element accessors table (1805-1849), including `the`'s
  ceiling-1 requirement (1838-1842); §4.4 selection tiebreak — "first element in ascending id byte
  order" (1276-1280).
- `docs/reference/determinism-contract.rst` — intra- vs. cross-implementation determinism (30-65,
  libm/sigmoid non-reproducibility named explicitly); the RNG chapter (1026-1133) — pinned `ChaCha8Rng`
  algorithm, per-carrier stream layout, and the **R8 declaration that Python's RNG streams are a closed
  epoch** re-blessed by ensemble-envelope comparison, not byte replay (1110-1133).
- `rust/crates/babylon-kernel/src/rng.rs` (full file read) — `KernelRng::for_carrier`, `seed_for`,
  `next_f64`; deliberately **no tick-global or entropy-seeded constructor** (65-69).
- `rust/crates/babylon-bsl/src/evaluator.rs` — `EFFECT_POSITION_ONLY` (464-484, 19 verbs, includes
  `update-edge`), `UNSERVED_EXPRESSION_HEADS` (486-512, includes `edges`/`edge-between`/`the` — Slice 2),
  `SERVED_QUERY_HEADS` (514-527, `nodes`/`neighbors`), `EVALUATOR_SERVED` (2251-2262, 10 heads:
  and/or/not/if/field-of/fold/exists/forall/select-max/select-min), `eval_field_of` doc (1185-1191,
  `NodeRef` referents only, `EdgeRef` unreachable today).
- `rust/crates/babylon-bsl/src/structural_verbs.rs` — module doc (16-26) and the `update-edge`/
  `update-hyperedge` hard-refusal arm (693-700): "`GraphSubstrate` has no storage for either: its edge
  state is one `f64` strength keyed by `(type, from, to)`... a declared Phase-2/substrate decision."
- `rust/crates/babylon-graph/src/substrate.rs` — the `GraphSubstrate` trait: `add_edge` (111-117,
  mandatory scalar `strength: f64` at mint time only) and `remove_edge` (124) are the **entire** edge
  write surface; no `update_edge`/`set_edge_attribute` method exists on the trait at all.
- `rust/crates/babylon-bsl/src/intrinsic_host.rs` (full file read) — `KernelIntrinsicHost` (57-70)
  dispatches `"floor"` only; `{exp, log}` are declarable (`declarations.rs:110`) but not evaluator-
  dispatchable (independently verified, matching the sibling `survival-port-phase1-inventory`'s finding).
- `rust/crates/babylon-tick/content/scenarios/organization-foundation.bscn` (full file read) — the live
  `enum` deffield precedent: `(defenum OrgKind (STATE_APPARATUS BUSINESS POLITICAL_FACTION
  CIVIL_SOCIETY))` / `(deffield organization/kind enum OrgKind)`, and the edge-literal mint syntax
  `(edge EdgeType/SOLIDARITY reading-group precinct 1)` — one trailing scalar, matching `add_edge`'s
  signature exactly.

## 2. COMPUTATION CATALOG (execution order, `StruggleSystem.step`, struggle.py:278-457, then three
helper calls at 454-457)

### 0 — Legitimation backfire multiplier (`_legitimation_backfire_multiplier`, struggle.py:242-264,
called once at the top of `step()`, line 309)
- **(a)** If a government is seated (graph-level register present) AND the population-weighted mean
  territory legitimation has fallen below a floor, return a crisis amplifier ≥ 1; otherwise exactly 1.0.
- **(b)** Guard: `if not graph.get_graph_attr("electoral_governments", None): return 1.0` (254). Mean:
  `_mean_legitimation` (266-276) — `weighted += pop * legitimation_index; population += pop`, then
  `weighted / population if population > 0 else 0.5` (273-276). Threshold: `if mean >=
  legitimacy_backfire_threshold: return 1.0` (262-263). Amplifier (legitimation.py:82-86):
  `amplifier = 1.0 + (1.0 - mean_legitimation) * (scale - 1.0)`.
- **(c) Reads:** graph-level attr `electoral_governments` (dict, written by `ElectoralSystem` @17.45,
  **prior tick** relative to Struggle @16.0 — the code comment at struggle.py:305-308 states this
  explicitly); `TERRITORY.legitimation_index` (`float | None`, default-absent→0.5, territory.py:240-254)
  and `TERRITORY.population` on every territory node.
- **(d) Writes:** none (return value only).
- **(e) Defines:** `politics.legitimacy_backfire_threshold` (0.35, `[0,1]`, defines.yaml:1121);
  `consciousness.legitimation_amplifier_scale` (2.0, `[1.0,10.0]` — **out of `[0,1]`**, defines.yaml:653).
- **(f) Events:** none.
- **Dormancy:** `electoral_governments` is written only by `ElectoralSystem` for **party-bearing**
  scenarios; every canonical `SCENARIOS` factory (`_legacy.py`, `single_county.py`, `org_probe.py`)
  seeds **zero** organization/party nodes with electoral standing (grep-confirmed: no `SocialRole.`/
  party-object seeding matching an electoral register anywhere in those factories), matching the code's
  own comment: "so the six party-less qa scenarios are byte-identical: no register ⟹ the multiplier is
  exactly 1.0" (struggle.py:307-308). **This computation is provably `1.0` on every canonical scenario.**

### 1 — Main uprising loop (struggle.py:314-451, one iteration per `graph.query_nodes()` node with
`role ∈ {PERIPHERY_PROLETARIAT, LUMPENPROLETARIAT}`)
- **(a)** For each struggling-role, active, non-territory node: roll a Bernoulli "spark" scaled by
  repression; on a hit, emit `EXCESSIVE_FORCE` and add backfire agitation. Then check the uprising
  condition (spark OR P(S|R)>P(S|A), AND agitation over threshold); on a hit, destroy a fraction of
  wealth, increment `solidarity_strength` on every incoming `SOLIDARITY` edge, boost class consciousness
  by a **flat per-uprising amount** (not the actual edge deltas — see the subtlety below), and emit
  `UPRISING` (+ `SOLIDARITY_SPIKE` if any edge actually gained strength).
- **(b)** Spark: `spark_probability = repression * spark_scale` (342); `spark_occurred = rng.random() <
  spark_probability` (343, `rng = resolve_rng(services, tick)`, line 299 — Python's shared MT19937
  stream, one `random.Random` instance consumed once per iteration of the **unsorted** loop). Backfire:
  `backfire = repression_backfire * backfire_multiplier` (362); `_update_agitation(attrs, backfire)`
  (363) — `new_value = max(0.0, current + delta)` (struggle.py:161, floor-only clamp). Uprising
  condition: `revolutionary_pressure = p_rev > p_acq` (368); `uprising_condition = (spark_occurred or
  revolutionary_pressure) and (agitation > resistance_threshold)` (369-371). Wealth: `new_wealth =
  current_wealth * (1.0 - wealth_destruction)` (381). Solidarity: for every `EdgeType.SOLIDARITY` edge
  targeting this node, `new_strength = min(1.0, current_strength + solidarity_gain)` (393, **upper-only**
  clamp); `solidarity_gained += new_strength - current_strength` (400). **Subtlety (verbatim, transcribe
  as-is):** the consciousness boost at 405-407 is computed from `solidarity_gain`
  (`services.defines.struggle.solidarity_gain_per_uprising`, a per-tick constant read once at line 294)
  — **not** from the local `solidarity_gained` accumulator (385-401, the actual sum of this node's edge
  deltas). A node with **zero** incoming `SOLIDARITY` edges (`edges_updated=0`, `solidarity_gained=0.0`)
  still receives the identical `consciousness_boost = solidarity_gain *
  consciousness_solidarity_boost` as a node with many edges — the two same-family-named variables
  (`solidarity_gain` the define, `solidarity_gained` the local accumulator) are genuinely distinct, and
  the code uses the former. Consciousness write: `_update_class_consciousness` — `new_value = max(0.0,
  min(1.0, current + delta))` (97, **double-sided** clamp, a third clamp shape in this same file).
- **(c) Reads:** `SOCIAL_CLASS.role`, `.active` (default `True`), `.repression_faced` (default
  `services.defines.DEFAULT_REPRESSION_FACED` = 0.5), `.ideology.agitation` (default 0.0 via
  `_get_agitation_from_node`), `.p_acquiescence` (default 0.5), `.p_revolution` (default 0.0),
  `.wealth` (default 0.0), `.ideology.class_consciousness` (via `class_consciousness_from_node`);
  incoming `EdgeType.SOLIDARITY` edges' `solidarity_strength` (default 0.0).
- **(d) Writes:** `SOCIAL_CLASS.wealth`; `SOCIAL_CLASS.ideology` (the whole 3-field dict, twice per
  uprising — once for the backfire agitation bump at 363 via `_update_agitation`+no write [that call
  mutates a **local copy**, `attrs`, not the graph — see the defect note below — the actual graph write
  of the agitation bump happens implicitly only via re-read at 364, `agitation =
  _get_agitation_from_node(attrs)`, which reads the SAME local dict `_update_agitation` just wrote a
  KEY onto ... **wait, verbatim recheck:** `_update_agitation(attrs, backfire)` (363) returns a **new**
  dict and discards it — the return value is not assigned. `attrs` itself (the local node-attrs dict
  from `node.attributes`) is never mutated by that call (`_update_agitation` builds and returns a new
  dict, struggle.py:144-173; it does not mutate its `node_data` argument in place). Line 364's `agitation
  = _get_agitation_from_node(attrs)` therefore re-reads the **unchanged** `attrs["ideology"]["agitation"]`
  — the backfire delta computed at line 362-363 is **silently discarded** and never reaches the graph or
  the local `agitation` variable used in the uprising-condition check at line 370. **This is a verbatim
  defect in the frozen system: the EXCESSIVE_FORCE backfire agitation bump has no effect on this tick's
  own uprising-condition check, and (since `graph.update_node` is never called for it either) no effect
  on any later tick's reads of `ideology.agitation` — the backfire coefficient
  `consciousness.repression_backfire` and the whole `backfire_multiplier` computation (§0) are dead
  code on the agitation channel.** Port-as-is law: transcribe this defect verbatim, D-record it, do not
  silently fix it.]); `SOCIAL_CLASS.ideology` (the real write, for the uprising's own consciousness
  boost, line 409: `graph.update_node(node.id, ideology=new_ideology)`); `SOLIDARITY` edge
  `solidarity_strength` (394-399, `graph.update_edge(...)`).
- **(e) Defines:** `struggle.spark_probability_scale` (0.1, `[0,1]`, defines.yaml:280);
  `struggle.resistance_threshold` (0.1, `[0,1]`, defines.yaml:281); `struggle.wealth_destruction_rate`
  (0.05, `[0,1]`, defines.yaml:282); `struggle.solidarity_gain_per_uprising` (0.2, `[0,1]`,
  defines.yaml:283); `struggle.consciousness_solidarity_boost` (0.5, `[0,1]`, defines.yaml:284);
  `consciousness.repression_backfire` (0.3, `[0,1]`, defines.yaml:218) — **dead per the defect above**;
  `survival.default_repression` (0.5, `[0,1]`, defines.yaml:167, via `DEFAULT_REPRESSION_FACED`).
- **(f) Events:** `EXCESSIVE_FORCE` (347-357, unconditional on the spark roll), `UPRISING` (417-436,
  unconditional on the uprising condition), `SOLIDARITY_SPIKE` (440-451, conditional on
  `solidarity_gained > 0`).

### 2 — George Jackson power-vacuum bifurcation (`_check_power_vacuum`, struggle.py:523-587, plus
`_apply_revolutionary_offensive` 589-624 and `_apply_fascist_revanchism` 626-675; called at 454)
- **(a)** Find the (first) `COMPRADOR_BOURGEOISIE` node; if none, or if it is solvent, stop. Find the
  (first) `PERIPHERY_PROLETARIAT` node; if none, stop. Compute revolutionary capacity =
  `organization × class_consciousness`; emit `POWER_VACUUM`; branch on the `jackson_threshold`.
- **(b)** `comprador = _find_entity_by_role(graph, COMPRADOR_BOURGEOISIE)` (542, **first match in
  unsorted `graph.query_nodes()` iteration order** — §5); insolvency test: `if wealth >= subsistence:
  return` (551); `revolutionary_capacity = organization * consciousness` (564, `organization` default
  0.1, `consciousness` via `class_consciousness_from_node`). Revolutionary Offensive branch (≥
  threshold): `graph.update_node(p_w_id, p_revolution=1.0, ideology=new_ideology)` (608, `new_ideology`
  from `_update_agitation(p_w_data, revolutionary_agitation_boost)`, floor-only clamp). Fascist
  Revanchism branch (< threshold): find `LABOR_ARISTOCRACY` (first match, same ordering caveat);
  `new_acq = min(1.0, current_acq + fascist_acquiescence_boost)` (656, **upper-only** clamp, a fourth
  clamp instance); `graph.update_node(c_w_id, ideology=new_ideology, p_acquiescence=new_acq)` (658,
  `new_ideology` from `_update_national_identity`, double-sided clamp).
- **(c) Reads:** `SOCIAL_CLASS.wealth`, `.subsistence_threshold` (default 5.0), `.organization` (default
  0.1), `.ideology.class_consciousness`, `.ideology.agitation`, `.ideology.national_identity` (default
  0.5), `.p_acquiescence` (default 0.5) — across three distinct role-filtered node lookups.
- **(d) Writes:** `SOCIAL_CLASS.p_revolution` (hard-set 1.0, periphery, offensive branch only);
  `SOCIAL_CLASS.ideology` (periphery agitation OR core-worker national-identity, branch-dependent);
  `SOCIAL_CLASS.p_acquiescence` (core worker, revanchism branch only).
- **(e) Defines:** `struggle.jackson_threshold` (0.4, `[0,1]`, defines.yaml:285);
  `struggle.revolutionary_agitation_boost` (0.5, `[0.0, 2.0]` — **out of `[0,1]`, the same D-1-class
  domain hazard Territory's `rent_spike_multiplier` hit**, defines.yaml:286);
  `struggle.fascist_identity_boost` (0.2, `[0,1]`, defines.yaml:287);
  `struggle.fascist_acquiescence_boost` (0.2, `[0,1]`, defines.yaml:288).
- **(f) Events:** `POWER_VACUUM` (567-579, unconditional once a solvent-comprador+periphery pair is
  found), then exactly one of `REVOLUTIONARY_OFFENSIVE` (611-624) or `FASCIST_REVANCHISM` (661-675).
- **RESERVED-LINE note:** this whole computation names and mechanizes the George Jackson Bifurcation
  (comprador insolvency → labor-aristocracy/periphery class-capacity fork) — the National-Question/
  labor-aristocracy theoretical content the Constitution reserves to the Director (MLM-TW doctrine,
  political framing). No parameter here cites ADR171's bribe:deprivation=1.55 ratio directly (grepped
  `struggle.py`, `survival.py`'s `StruggleDefines`, and the `struggle:`/`consciousness:` blocks of
  `defines.yaml` — zero hits), but the `national_identity` axis boosted here (fascist-revanchism branch)
  and the `SocialRole` taxonomy itself (`COMPRADOR_BOURGEOISIE`/`LABOR_ARISTOCRACY`/
  `PERIPHERY_PROLETARIAT`) are the same theoretical apparatus. Described here, not touched.

### 3 — Peripheral revolt (`_check_peripheral_revolt`, struggle.py:677-748, called at 455)
- **(a)** If the (first) `PERIPHERY_PROLETARIAT` node has P(S|R) strictly greater than P(S|A), sever
  every outgoing `EXPLOITATION` edge from it and emit `PERIPHERAL_REVOLT`.
- **(b)** `if p_rev <= p_acq: return` (712, strict-inequality gate — a tie or P(S|A)-favoring case is
  quiet, no event, no partial severing). Edge collection: `for edge in graph.query_edges(edge_type=
  EdgeType.EXPLOITATION): if edge.source_id == p_w_id: edges_to_remove.append(...)` (718-720) — then a
  **second pass** removes them one at a time (722-724, "protocol has no batch remove" per the inline
  comment at 722). Event payload decoration reads a **third** graph-level register: `opposition_states =
  graph.get_graph_attr("opposition_states", {}) or {}` (730) — `capital_labor_gap` is included purely
  for narrative/observer context and does **not** gate the revolt condition (the comment at 726-729 is
  explicit: "no change to the revolt CONDITION").
- **(c) Reads:** `SOCIAL_CLASS.active`, `.p_acquiescence`, `.p_revolution` (all on the periphery node
  only); every `EdgeType.EXPLOITATION` edge in the graph (filtered by `source_id` in Python, not by a
  graph-side directed query); graph-level attr `opposition_states` (dict, written by `ContradictionSystem`
  @18.0, **prior tick** relative to Struggle @16.0).
- **(d) Writes:** removes N `EXPLOITATION` edges (structural, not an attribute write).
- **(e) Defines:** none (no coefficient reads in this computation).
- **(f) Events:** `PERIPHERAL_REVOLT` (732-748, unconditional once the revolt condition and edge
  collection complete — fires even if `edges_to_remove` is empty, e.g. a periphery node with no outgoing
  EXPLOITATION edges left from a prior revolt; `edges_severed` would read 0).

### 4 — Spontaneous riot (`_check_spontaneous_riot`, struggle.py:459-521, called at 457)
- **(a)** For every `LUMPENPROLETARIAT` node (deterministic gate, no RNG — the docstring at 469-470
  states this explicitly: "a gate, not a stochastic roll"), if `volatility × (1 − discipline)` exceeds a
  threshold, destroy wealth and emit `SPONTANEOUS_RIOT`. Builds **no** solidarity infrastructure — "the
  reactionary inverse of the George Floyd dynamic" (struggle.py:472).
- **(b)** `for node in sorted(graph.query_nodes(node_type=NodeType.SOCIAL_CLASS), key=lambda n: n.id)`
  (480, **the one place in this file that sorts iteration by node id** — see §5 for the contrast with
  every other loop in this system). `riot_risk = calculate_spontaneous_riot_risk(volatility, discipline)`
  (495-497, reactionary.py:116: `risk = volatility * (1.0 - discipline); return max(0.0, min(1.0, risk))`
  — double-sided clamp). `if riot_risk <= threshold: continue` (498, boundary-inclusive skip — strictly
  `>` threshold fires). `new_wealth = current_wealth * (1.0 - wealth_destruction)` (502, the **same**
  `struggle.wealth_destruction_rate` define used by computation 1's uprising branch).
- **(c) Reads:** `SOCIAL_CLASS.active`, `.role`, `.volatility` (default 0.0), `.organization` (read here
  as the "organizational discipline" proxy — default 0.0, **not** `services.defines.DEFAULT_ORGANIZATION`
  = 0.1 used elsewhere; a distinct absent-default choice worth transcribing verbatim), `.wealth`.
- **(d) Writes:** `SOCIAL_CLASS.wealth` only.
- **(e) Defines:** `reactionary.spontaneous_riot_threshold` (0.5, `[0,1]`, defines.yaml:938);
  `struggle.wealth_destruction_rate` (0.05, `[0,1]`, same define as computation 1).
- **(f) Events:** `SPONTANEOUS_RIOT` (504-521, unconditional once the risk gate passes).
- **Dormancy:** **self-declared dormant in the frozen source** (struggle.py:473: "The canonical world
  seeds no lumpen nodes, so this branch is dormant there") — independently confirmed: zero
  `SocialRole.LUMPENPROLETARIAT` seeds anywhere in `src/babylon/engine/scenarios/*.py` (grepped every
  `SocialRole.` use across the scenario-factory tree).

**Events emitted by the whole system: 8 distinct `EventType` members** (`EXCESSIVE_FORCE`, `UPRISING`,
`SOLIDARITY_SPIKE`, `POWER_VACUUM`, `REVOLUTIONARY_OFFENSIVE`, `FASCIST_REVANCHISM`, `PERIPHERAL_REVOLT`,
`SPONTANEOUS_RIOT`), 8 `services.event_bus.publish(Event(...))` call sites (struggle.py:347, 419, 442,
506, 569, 613, 663, 734), one publish per event type — grep-confirmed no other `EventType.` reference in
the file. Per the CURRENT BSL surface: `TickReport` carries no event log, so all 8 are WS1 (#502) ledger
rows, unpinnable by goldens today.

## 3. TYPE INVENTORY

Runtime storage note (load-bearing, matching the Territory/Survival inventories' finding):
`BabylonGraph.update_node`/`.update_edge` (`topology/graph.py:660-670,690`) are plain dict merges with
no type coercion or quantization mid-tick. The `SnapToGrid` 1e-5 grid (`models/types.py`) applies only
at Pydantic model *instantiation*.

| Attribute | Node/edge type | Python model type | Domain | Category |
|---|---|---|---|---|
| `role` | SOCIAL_CLASS | `SocialRole` (StrEnum, 8 members; struggle.py touches 4) | closed set | **Enum discriminant** |
| `active` | SOCIAL_CLASS | `bool` | `{T,F}` | boolean |
| `wealth` | SOCIAL_CLASS | `Currency` (`Annotated[float, ge=0.0]`, SnapToGrid) | `[0,∞)` | **unbounded real, money-semantic** |
| `subsistence_threshold` | SOCIAL_CLASS | `Currency` | `[0,∞)` | **unbounded real, money-semantic** |
| `organization` | SOCIAL_CLASS | `Probability` (`Annotated[float, ge=0,le=1]`, SnapToGrid) | `[0,1]` | unit-interval |
| `repression_faced` | SOCIAL_CLASS | `Probability` | `[0,1]` | unit-interval |
| `p_acquiescence` | SOCIAL_CLASS | `Probability` | `[0,1]` | unit-interval — **written by `SurvivalSystem` via a stipulated sigmoid same-tick, and hard-overwritten by this system in the fascist-revanchism branch** |
| `p_revolution` | SOCIAL_CLASS | `Probability` | `[0,1]` | unit-interval — **hard-set to exactly `1.0` by this system's revolutionary-offensive branch; no other engine System reads `p_revolution` downstream (grep-confirmed — only projection/persistence/observer layers do)** |
| `volatility` | SOCIAL_CLASS | `Intensity` (`Annotated[float, ge=0,le=1]`, SnapToGrid) | `[0,1]` | unit-interval, role-defaulted (`LUMPENPROLETARIAT`=0.8) |
| `ideology.class_consciousness` | SOCIAL_CLASS (nested) | plain `Annotated[float, Field(ge=0,le=1)]` — **NOT** the shared `Probability`/`Coefficient` type alias, **no `SnapToGrid`** | `[0,1]` | unit-interval, **never grid-quantized at any point** |
| `ideology.national_identity` | SOCIAL_CLASS (nested) | plain `Annotated[float, Field(ge=0,le=1)]`, no SnapToGrid | `[0,1]` | unit-interval, never grid-quantized |
| `ideology.agitation` | SOCIAL_CLASS (nested) | plain `Annotated[float, Field(ge=0.0)]`, no SnapToGrid | `[0,∞)` | **unbounded real, never grid-quantized** |
| `solidarity_strength` | edge, `SOLIDARITY` | plain `dict` attribute value (`edge.attributes.get(...)`, no Pydantic model enforces a domain on the edge-attribute dict at all — `GraphEdge.attributes: dict[str, Any]`) | `[0,1]` **by convention only** (every writer clamps to `≤1.0`; no declared type enforces it) | unit-interval by convention, structurally unenforced |
| `legitimation_index` | TERRITORY | `float \| None` (plain, not a type alias, no SnapToGrid) | `[0,1]` per docstring; `None`→every reader defaults to 0.5 | unit-interval, honest-null pattern |
| `electoral_governments` | graph-level | `dict` (via `get_graph_attr`) | — | graph-scope register, no BSL representation today (§5/§6) |
| `opposition_states` | graph-level | `dict` (via `get_graph_attr`) | — | graph-scope register, event-payload decoration only |
| `spark_probability_scale`, `resistance_threshold`, `wealth_destruction_rate`, `solidarity_gain_per_uprising`, `consciousness_solidarity_boost`, `jackson_threshold`, `fascist_identity_boost`, `fascist_acquiescence_boost`, `repression_backfire`, `spontaneous_riot_threshold`, `legitimacy_backfire_threshold`, `default_repression` (defines) | — | `float` | `[0,1]` | unit-interval coefficients |
| `revolutionary_agitation_boost` (define) | — | `float` | `[0.0, 2.0]` | **out-of-`[0,1]` coefficient, D-1-class hazard (§6)** |
| `legitimation_amplifier_scale` (define) | — | `float` | `[1.0, 10.0]` | **out-of-`[0,1]` coefficient, D-1-class hazard (§6)** |

**Ideology-flattening finding (new, not present in the Territory/Survival inventories):** `ideology` is
a nested 3-field sub-model (`IdeologicalProfile`) written as one JSON-shaped blob via
`graph.update_node(id, ideology={...})`. BSL's `deffield` vocabulary has no nested/object field type —
every field is a flat scalar (`bsl-language.rst` §2.9). Every one of struggle.py's five `ideology=...`
writes (409, 608, 658, plus the two dead-code writes inside `_update_agitation`/`_update_national_identity`
that never reach the graph per the §2 defect note) would need to flatten into three independent scalar
`deffield`s (e.g. `social-class/class-consciousness`, `social-class/national-identity`,
`social-class/agitation`), each written by its own `update-node` verb call (BSL's `update-node` grammar
is one field per call — `<verb> ::= "(" "update-node" <expr> <qname> <update-op> ")"`,
bsl-language.rst:1330 — never a combined multi-field write like Python's kwargs call). Mechanical, not
blocking, but touches every write in this system — D-record candidate.

**Enum-discriminant flag — `role` has a real, already-landed precedent, unlike Territory's `profile`/
`territory_type` finding.** `organization-foundation.bscn` (read in full) already declares and seeds a
content enum exactly this shape: `(defenum OrgKind (STATE_APPARATUS BUSINESS POLITICAL_FACTION
CIVIL_SOCIETY))` / `(deffield organization/kind enum OrgKind)`. `SocialRole` (8 members, struggle.py
reads 4 of them) maps directly onto the same `defenum`+`deffield ... enum` pattern (ADR195/196, landed).
This is **portable now**, not a D-record-requiring workaround — a correction, in the favorable direction,
to what Territory's inventory found for its own two enums (which predate this precedent).

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `float`); **zero libm transcendentals in struggle.py itself**
— grep-confirmed no `exp`/`log`/`math.` reference anywhere in the file, and the one formula it calls
(`calculate_spontaneous_riot_risk`, reactionary.py:94-117) and the one domain function it calls
(`compute_legitimation_amplifier`, legitimation.py:36-86) are both pure linear/clamp arithmetic, no
libm. Shapes, in execution order:

1. **Population-weighted mean + guarded division:** `_mean_legitimation` — `weighted += pop *
   legitimation; population += pop`, then `weighted/population if population>0 else 0.5` (struggle.py:
   270-276) — a manual `fold sum` pair with a divide-by-zero guard.
2. **Linear affine amplifier:** `1.0 + (1.0 - mean_legitimation) * (scale - 1.0)` (legitimation.py:84) —
   two subtracts, two multiplies, one add. Bare `1.0` literals throughout (×3) — the same "no bare
   non-integer literal" BSL-parser issue Territory's inventory flagged.
3. **Multiplicative probability:** `repression * spark_scale` (struggle.py:342) — one multiply.
4. **Stochastic comparison:** `rng.random() < spark_probability` (343) — **not a libm hazard, but a
   distinct nondeterminism/port-fidelity category: a shared-stream Mersenne-Twister draw consumed in
   UNSORTED graph-iteration order.** R8/`determinism-contract.rst` (1110-1133) rules Python's RNG
   streams a closed epoch and pins a structurally different Rust design (`KernelRng`, per-carrier
   `(session_id, tick, domain, stable_key)` streams, grain-invariant by construction — `rng.rs` docs
   1-33) — byte-parity with this exact draw is explicitly not the reconciliation target; ensemble-
   envelope comparison is.
5. **Additive floor-clamp:** `_update_agitation` — `max(0.0, current + delta)` (161) — used for the
   backfire bump (discarded per the §2 defect), the revolutionary-offensive agitation boost (608, this
   one DOES reach the graph).
6. **Multiplicative decay, bare `1.0`:** `current_wealth * (1.0 - wealth_destruction)` (381, uprising)
   and `current_wealth * (1.0 - wealth_destruction)` (502, spontaneous riot) — **identical shape, same
   define, two call sites** — both carry the bare-`1.0`-literal parser issue.
7. **Additive upper-only clamp:** `min(1.0, current_strength + solidarity_gain)` (393, SOLIDARITY edge)
   and `min(1.0, current_acq + fascist_acquiescence_boost)` (656, p_acquiescence) — **the same
   upper-only clamp shape Territory's Phase-3 spillover used** (`territory.py:315`), now appearing twice
   more in a different system — this is a recurring pattern across the estate, not a Struggle-specific
   oddity.
8. **Additive double-sided clamp:** `max(0.0, min(1.0, current + delta))` — `_update_class_consciousness`
   (97), `_update_national_identity` (129), and (independently, same shape) `calculate_spontaneous_riot_risk`'s
   own `max(0.0, min(1.0, risk))` (reactionary.py:117). **Three clamp shapes now confirmed present in one
   system** (floor-only §5 / upper-only §7 / double-sided §8) — the same "transcribe each shape
   faithfully, do not unify" port-as-is law Territory's inventory established.
9. **Comparison chain (boolean logic, no arithmetic):** `revolutionary_pressure = p_rev > p_acq` (368);
   `uprising_condition = (spark_occurred or revolutionary_pressure) and (agitation > resistance_threshold)`
   (369-371); `if p_rev <= p_acq: return` (712, strict-inequality gate, peripheral revolt); `if riot_risk
   <= threshold: continue` (498, boundary-inclusive skip).
10. **Multiplicative capacity:** `organization * consciousness` (564) — one multiply, feeds the
    `jackson_threshold` comparison.
11. **Multiplicative risk:** `volatility * (1.0 - discipline)` (reactionary.py:116) — same decay shape
    as item 6, different define family.
12. **Running-total subtraction (no clamp):** `solidarity_gained += new_strength - current_strength`
    (400) — unclamped; can legitimately be `0.0` if the edge was already at its `1.0` ceiling before this
    uprising (the SOLIDARITY_SPIKE gate at 439 correctly treats that as "no spike").

**No Real→Int demotions anywhere in this system** (unlike Territory's population-displacement/camp-decay
sites) — every quantity here stays `float`-typed end to end; `population`/`organization` are read but
never divided-then-truncated within struggle.py's own arithmetic.

**`libm_hazards` verdict for this system's own code: FALSE.** The one adjacent hazard — `SurvivalSystem`'s
`calculate_acquiescence_probability` (`formulas/survival_calculus.py:41-43`, `1.0/(1.0+math.exp(exponent))`,
a stipulated logistic sigmoid, ADR172/173 "PORT-QUESTION" territory) writes `p_acquiescence` in the SAME
tick, one position earlier (15.0 < 16.0), and struggle.py reads that value at line 338 — this is a
cross-system CHANNEL hazard (§5), not this system's own arithmetic. Independently verified: `exp` is
declarable/typechecks (`declarations.rs:110`) but **has no evaluator dispatch today**
(`KernelIntrinsicHost::call`, `intrinsic_host.rs:57-70`, matches only `"floor"`) — so even setting aside
the ADR172/173 functional-form question, the sigmoid Struggle depends on is doubly blocked upstream on
its own, independent axis.

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 16.0** (struggle.py:235), confirmed against `_SYSTEM_CLASSES`
  (`simulation_engine.py:328-364`): `... DoctrineSystem(14.7) → SurvivalSystem(15.0) →
  StruggleSystem(16.0) → ConsciousnessSystem(17.0) → FascistFactionSystem(17.4) →
  AllegianceSystem(17.42) → ElectoralSystem(17.45) → PolicySystem(17.47) → SovereigntySystem →
  MarketScissorsSystem(17.8) → ContradictionSystem(18.0) → ...`.
- **Reads from same-tick prior systems:** `SurvivalSystem` @15.0 writes `p_acquiescence`/`p_revolution`
  every tick for every active non-territory node (`survival.py:165`) — Struggle reads both, same tick,
  immediately downstream (§4's sigmoid-channel hazard). `ElectoralSystem` @17.45 writes
  `electoral_governments` — Struggle @16.0 reads it, so it always sees the **prior tick's** value
  (confirmed by the code's own comment, struggle.py:305-308). `ContradictionSystem` @18.0 writes
  `opposition_states` — same prior-tick relationship for the peripheral-revolt event payload.
- **Writes consumed later this tick / downstream ticks:**
  - `SOCIAL_CLASS.wealth` — the single most load-bearing channel (grep-confirmed reads in
    `ConsciousnessSystem`(17.0, same tick), `AllegianceSystem`(17.42, same tick),
    `MarketScissorsSystem`(17.8, same tick), `ContradictionSystem`(18.0, same tick),
    `ContradictionFieldSystem`(19.0, same tick), `EdgeTransitionSystem`(21.0, same tick), and — next
    tick, positions < 16 — `VitalitySystem`, `ProductionSystem`, `ImperialRentSystem`, `DecompositionSystem`,
    `SurvivalSystem` itself.
  - `SOCIAL_CLASS.ideology` (dict) — read downstream by `ConsciousnessSystem`(17.0, same tick, the
    primary consumer/updater of `IdeologicalProfile`), `FascistFactionSystem`(17.4, same tick, via
    `reactionary.py:351`), and — next tick — `SolidaritySystem`(8.0).
  - `SOLIDARITY` edge `solidarity_strength` — read downstream by `SurvivalSystem`(15.0, next tick, the
    organization multiplier), `SolidaritySystem`(8.0, next tick, the transmission mechanism itself),
    `CommunitySystem`(6.0, next tick), `DoctrineSystem`(14.7, next tick, decay), and — same tick, after
    Struggle — `FascistFactionSystem`(17.4), `ElectoralSystem`(17.45), `PolicySystem`(17.47). This is a
    **7-system channel**, structurally the SOLIDARITY-edge analogue of Territory's `population` finding
    — and it is the exact channel the `update-edge` substrate gap (§6) blocks for every one of those 7
    systems, not just this one.
  - `SOCIAL_CLASS.p_revolution` — **read downstream by NO other engine System** (grep-confirmed: the
    only other `p_revolution` hits in `src/babylon/` are projection/persistence/observer/metrics layers,
    never another System's `step()`). A terminal/observational output from the engine's own point of
    view, **except** that `SurvivalSystem` recomputes and overwrites it unconditionally every tick
    (`survival.py:165`) — so the revolutionary-offensive branch's `p_revolution=1.0` override survives
    only from Struggle@16.0 this tick through `EpistemicHorizonSystem`@22.0 this tick and any
    position-<15 systems next tick, before `SurvivalSystem`@15.0 next tick recomputes it fresh via the
    sigmoid formula.
  - `SOCIAL_CLASS.p_acquiescence` — read downstream by exactly one engine System,
    `EpistemicHorizonSystem`@22.0 (same tick, `epistemic_horizon.py:95`), before `SurvivalSystem`@15.0
    next tick overwrites it the same way.
  - `EXPLOITATION` edges (removed by peripheral revolt) — read downstream by `ImperialRentSystem`(9.0,
    next tick, the extraction pipeline itself — the removal is the mechanism by which "no extraction →
    no super-wages → LA decomposition" per the class docstring), `ContradictionSystem`(18.0, same tick),
    `ContradictionFieldSystem`(19.0, same tick).
- **Iteration-order inconsistency within this one file (new finding, no Territory/Survival precedent):**
  `_check_spontaneous_riot` explicitly sorts (`sorted(..., key=lambda n: n.id)`, struggle.py:480) —
  matching BSL's own `select-max`/`select-min` tiebreak convention ("first element in ascending id byte
  order," bsl-language.rst:1277) — but the main uprising loop (314), `_find_entity_by_role` (176-210,
  used 3× by `_check_power_vacuum`/`_check_peripheral_revolt`), and the EXPLOITATION-edge collection loop
  (718) all consume **unsorted** `graph.query_nodes()`/`.query_edges()` iteration, which is rustworkx
  insertion order (`query_mixin.py:50`), not id order. For the main loop this is state-affecting (the
  shared RNG stream's draws land on whichever node insertion order visits first); for
  `_find_entity_by_role` it is currently inert because every canonical scenario seeds exactly one node
  per relevant role (`_legacy.py:83,313,328,358` — `create_two_node_scenario`/`create_imperial_circuit_scenario`,
  the base factory every other canonical scenario name in `tools/regression_scenarios.py`'s `SCENARIOS`
  dict wraps via `inject_parameter` overrides) — but it is a real, precisely-named latent divergence from
  BSL's ordering discipline the moment a scenario ever seeds two nodes with the same role.
- **Context/service usage with no BSL equivalent:** `graph.get_graph_attr("electoral_governments", ...)`
  (254) and `graph.get_graph_attr("opposition_states", ...)` (730) are both **graph-scope state** — R9
  chapter C3's ratified answer (bsl-language.rst:2650-2679) is to model each as an ordinary `deffield` on
  a declared ceiling-1 carrier `NodeType`, read via `(the NodeType/...)` — but `the` itself is
  UNSERVED (Slice 2, `evaluator.rs:506`), and no carrier `NodeType` for either register exists in the
  closed vocabulary today (adding one is its own amendment-territory decision per that same ruling). For
  `electoral_governments` this is moot in practice (§2's Computation 0 dormancy finding: provably `1.0`
  on every canonical scenario) — declare `:const 1.0c` and file the machinery to WS1, the same move
  Territory's inventory made for `TickContext.displacement_mode`. For `opposition_states` the read only
  decorates an event payload (never gates behavior), so its absence costs narrative richness, not
  simulation correctness.
- **RNG service usage with no BSL equivalent:** `resolve_rng(services, tick)` (struggle.py:299,
  `system_base.py:35-55`) — Python's shared, tick-seeded (not session-seeded) `random.Random` stream. No
  BSL construct reaches it or any equivalent — RNG draws are categorically prohibited inside BSL rule
  content (§6).

## 6. BLOCKER ASSESSMENT (adjudicated against the CURRENT BSL surface, dev tree)

| Computation | Verdict | Detail |
|---|---|---|
| Legitimation backfire multiplier (struggle.py:242-264) | **PORTABLE WITH D-RECORD** | Provably `1.0` on every canonical scenario (§2 Computation 0) — declare `:const 1.0c`, Metabolism-D-2/Territory-`displacement_mode`-style "provably uniform" reasoning; file the `electoral_governments`/`the`-accessor/carrier-NodeType machinery to the WS1 ledger. If a future party-bearing scenario needs the live path, that path is separately BLOCKED on the R9-chapter-C3 graph-scope-state pattern (needs a declared carrier `NodeType` — amendment territory — plus `the`'s Slice-2 landing). |
| Main loop — spark roll + backfire (struggle.py:341-364) | **BLOCKED — RNG-as-BSL-content-primitive, categorically prohibited** | `bsl-language.rst`'s §2.8 "Prohibited" clause is explicit and permanent, not a not-yet-built lane: "no randomness primitive (RNG draws are **kernel intrinsics**...)" (1620-1624). This is architecturally different from every other blocker this port program has filed to date — the fix is not "wait for a slice to land," it is either (a) implement the spark mechanic as Rust-native kernel code outside BSL content (a different porting strategy than every other landed pack), or (b) a language amendment exposing a kernel-intrinsic RNG draw as a rule-readable value. Separately, the backfire bump itself is a **verbatim dead-code defect** in the frozen system (§2 Computation 1's discard-the-return-value bug) — port-as-is transcribes the defect, does not fix it. |
| Main loop — uprising condition (struggle.py:366-374) | **PORTABLE NOW** | Pure comparisons over already-computed values (`p_rev`, `p_acq`, `agitation`, `spark_occurred`) — trivially expressible as a rule guard once the spark input it depends on is available (chained to the RNG blocker above, not independently blocked). |
| Main loop — wealth destruction (struggle.py:381-382) | **PORTABLE NOW** | `current_wealth * (1.0 - wealth_destruction)`, bare `1.0`-literal only (the `c`-suffixed-`defconst` precedent every landed pack already uses) — chained to the uprising condition, not independently blocked. |
| Main loop — SOLIDARITY-edge solidarity-strength gain (struggle.py:388-401) | **BLOCKED — `update-edge` substrate storage, a declared gap, not a query-lane gap** | `update-edge` IS grammar-recognized (`EFFECT_POSITION_ONLY`, evaluator.rs:468) but hard-refused at effect-collection time: `GraphSubstrate` has no per-edge attribute storage beyond the single mandatory `strength: f64` set once at `add_edge` mint time (`substrate.rs:111-117`) — no `update_edge`/`set_edge_attribute` trait method exists at all (`structural_verbs.rs:693-700`, "a declared Phase-2/substrate decision"). This blocks not only Struggle but every one of the 7 systems that read/write `solidarity_strength` (§5). Reading the SOLIDARITY edge's current strength is additionally blocked on Slice 2 (`edge-between`/`field-of`-over-`EdgeRef`, unreachable today per `evaluator.rs:1186`) — a second, independent gate on top of the write gap. |
| Main loop — class-consciousness boost (struggle.py:404-409) | **PORTABLE WITH D-RECORD** | Pure arithmetic on a define constant (chained to the SOLIDARITY-edge write only for the SOLIDARITY_SPIKE *gating condition*, not for its own value — per the §2 subtlety, the boost itself never reads the edge deltas) — needs the `ideology`-dict-flattening D-record (§3) but no query-lane or substrate blocker of its own. |
| George Jackson bifurcation core (struggle.py:542-587, `organization × class_consciousness` + threshold) | **PORTABLE WITH D-RECORD** | Pure arithmetic + role-filtered node lookup. `role`-as-enum is portable now (§3, the `OrgKind` precedent); "find the first/only node of role X" needs a content-modeling D-record (either a `select-min`-by-id encoding using BSL's own ascending-id-byte-order tiebreak, or — cleaner — a per-role rule anchor with an `(when (= role RoleName))` guard, since canonical scenarios seed exactly one node per relevant role and the rule fires once naturally). `revolutionary_agitation_boost`'s `[0,2]` domain hits the same D-1-class hazard Territory's `rent_spike_multiplier` did (scale-op only helps a `Value::Currency` operand; these are `:field`-sourced `Value::Real`) — same accepted-deviation class under ADR183 §5.4. |
| Revolutionary Offensive write (struggle.py:606-608) | **PORTABLE WITH D-RECORD** | `_update_agitation` (floor clamp) + `p_revolution=1.0` hard-set — two `update-node` calls (grammar is one field per call), chained to the bifurcation core's D-records above, nothing independently new. |
| Fascist Revanchism write (struggle.py:643-658) | **PORTABLE WITH D-RECORD** | Same shape as Revolutionary Offensive — a second role-filtered lookup (`LABOR_ARISTOCRACY`) plus two `update-node` calls; the `ideology`-flattening D-record applies here too. |
| Peripheral revolt condition + edge severing (struggle.py:697-724) | **PORTABLE WITH D-RECORD, favorably reformulable** | Does **not** need edge-attribute reads: `remove-edge` takes `(EdgeType, from-expr, to-expr)` directly, not an `EdgeRef` (`substrate.rs:124`, `structural_verbs.rs`'s grammar), so `(for-each (neighbors periphery EdgeType/EXPLOITATION :out SOCIAL_CLASS) (remove-edge EdgeType/EXPLOITATION periphery target))` is expressible entirely within the LANDED Slice-1 surface (`neighbors` is a `SERVED_QUERY_HEAD`, `for-each`+`remove-edge` are `EFFECT_POSITION_ONLY`) — the same "favorably reformulable, collect-then-apply maps onto a pull-side query" finding Territory's inventory made for its own spillover phase. The `opposition_states` event-payload decoration is separately blocked on the same graph-scope-state gap as Computation 0, but does not gate the condition itself — omit it from a first pack and file it to WS1. |
| Spontaneous riot (struggle.py:480-521) | **PORTABLE NOW (modulo the `role`-enum D-record)** | Pure deterministic arithmetic (`calculate_spontaneous_riot_risk`, no libm), `role`-filtered `NodeType.SOCIAL_CLASS` query (already correctly enum-typed in the source, unlike the other two role lookups), a double-sided clamp already matching the landed nested-`if` convention, wealth destruction identical in shape to the main loop's. **Self-declared dormant on every canonical scenario** (§2), so its first conformance oracle is necessarily a hand-built `.bscn` fixture, not a harvested canonical vector. |
| `p_acquiescence`/`p_revolution`/`agitation` reads throughout (struggle.py:338-339, 337) | **PORT-QUESTION, not a blocker of Struggle's own arithmetic** | The values themselves come from `SurvivalSystem`'s stipulated logistic sigmoid (`math.exp`, ADR172/173-flagged) written the same tick, one position earlier. Struggle merely reads them — its own condition logic (`p_rev > p_acq`, `agitation > threshold`) is trivially portable arithmetic regardless of how P(S|A)/P(S|R) end up computed on the Rust side. Independently, `exp` has no evaluator dispatch today (`intrinsic_host.rs:57-70`) — doubly gated upstream, not by Struggle. |

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_struggle.py` | 667 (23 `def test_` functions) | **Primary conformance-oracle candidate.** Direct unit coverage of `StruggleSystem.step()`: spark probability/roll, uprising condition (all four boolean combinations), wealth destruction, SOLIDARITY-edge strength gain, class-consciousness boost, event emission for `EXCESSIVE_FORCE`/`UPRISING`/`SOLIDARITY_SPIKE`. |
| `tests/unit/engine/test_jackson_bifurcation.py` | 525 (15 `def test_` functions) | **Primary conformance-oracle candidate** for Computation 2: comprador-solvency gate, revolutionary-capacity threshold on both sides, the Revolutionary Offensive and Fascist Revanchism write shapes, `POWER_VACUUM`/`REVOLUTIONARY_OFFENSIVE`/`FASCIST_REVANCHISM` event payloads. |
| `tests/unit/engine/systems/test_struggle_volatility.py` | 81 (4 `def test_` functions) | Spec-071 spontaneous-riot coverage — Computation 4's dedicated unit tests (the dormant-on-canonical branch's only exercise today). |
| `tests/unit/formulas/test_reactionary.py` | 142 | Unit coverage of `calculate_spontaneous_riot_risk` in isolation (the formula Computation 4 calls) — a genuinely reusable conformance-oracle candidate independent of the System wiring. |
| `tests/unit/bifurcation/test_legitimation.py` | 293 | Unit coverage of `compute_legitimation_amplifier` (the formula Computation 0's live-path branch calls) — reusable independent of `StruggleSystem`. |
| `tests/unit/kernel/test_node_access.py` | 70 | Unit coverage of `class_consciousness_from_node` — the shared accessor 3 systems (including this one) call. |
| `tests/integration/mechanics/test_george_floyd_dynamic.py` | 528 | Integration-level narrative-scenario coverage of the Spark→Uprising pipeline end to end — behavioral-contract candidate (pins *what the system does*, not implementation), closer to a golden-scenario shape than the unit file. |
| `tests/integration/mechanics/test_class_struggle.py` | 737 | Broader integration coverage spanning Struggle alongside neighboring systems (Survival/Consciousness) — mixed conformance/narrative; would need triage to separate byte-level-oracle-worthy assertions from scenario narrative checks. |
| `tests/unit/engine/laws/` | — | **No `test_law_struggle_system.py` or equivalent exists** (searched `tests/unit/engine/laws/` directory listing for a struggle-named file — none found) — unlike Territory, this system has **no dedicated property-based invariant-law test file** today; the closest analogues (agitation non-negativity, consciousness/national-identity `[0,1]` bounds, the eviction-flag-style one-way `p_revolution=1.0` latch) are asserted inline inside `test_struggle.py`/`test_jackson_bifurcation.py`, not factored into a standalone laws module. |
| `src/babylon/models/events/struggle_payloads.py` | 116 | Not a test file, but the frozen Pydantic mirrors of 5/8 event payloads — a schema-level contract surface for what each event's fields must look like, useful for scoping WS1's eventual event-ledger shape. |

**qa:regression byte-gate coverage.** `tools/regression_test.py::graph_content_hash` hashes every
node/edge attribute of the `WorldState→graph` projection on every canonical scenario, so any change to
`StruggleSystem`'s outputs is caught by the byte-identical hash gate on the scenarios where it is live.
Per §2/§6's dormancy findings, that live coverage is real for: the main uprising loop (spark/backfire/
uprising/solidarity/consciousness — `tools/regression_scenarios.py:355-358,780` declares "the stochastic
EXCESSIVE_FORCE spark rolls every tick from repression_faced" as an explicit coverage claim on the
`imperial_circuit`/`glut` scenarios) and the George Jackson bifurcation's Fascist Revanchism branch
(`tools/regression_scenarios.py:1652-1658` declares it explicitly on the `fascist_bifurcation` scenario).
**Zero canonical coverage** for: the Revolutionary Offensive branch (no coverage-claim row names it
anywhere in the file — grepped `REVOLUTIONARY_OFFENSIVE`/`revolutionary_offensive` across the whole
file, no hits), peripheral revolt (`PERIPHERAL_REVOLT` — no hits), spontaneous riot (self-declared
dormant, §2/§6). A port's conformance fixtures for those three branches will need to be hand-built
`.bscn` scenarios, the same pattern Territory's and Metabolism's inventories established, not harvested
from the canonical estate.

---

## Adjudication (2026-08-12)

Adjudicated against the **current dev tree** (`9324482f`). The computation catalog is meticulous
and its headline defect finding — the discarded `_update_agitation` return — is genuine, newly
recorded, and re-verified line by line below. The verdict **BLOCKED stands**, but **both named
blockers are mis-stated**: one is not a categorical prohibition, and the other is missing the gate
that actually fires first. The single row graded portable in the blocked half is the one row that
is not.

1. **CORRECTION — RNG is NOT "categorically prohibited" in BSL content. It is a sanctioned kernel
   intrinsic whose signature is unbuilt — the same category as the `exp` gap this inventory names
   two rows later.** §6's spark-roll row reads the §2.8 Prohibited clause as "explicit and
   permanent, not a not-yet-built lane," and offers only two paths: Rust-native kernel code
   outside content, or a language amendment. Read the whole sentence at
   `docs/reference/bsl-language.rst:1620-1624`: "There is no I/O, no time source other than a
   `:tick` binding, no randomness primitive (**RNG draws are kernel intrinsics with the kernel's
   per-(session, tick, salt) seeding**, specified in :doc:`/reference/determinism-contract`)." The
   parenthetical is a sanction, not a carve-out: what is prohibited is a randomness *primitive*
   (a language form), while RNG *as a kernel intrinsic* is expressly provided for — and content
   calls kernel intrinsics routinely; `floor` is one, dispatched at `intrinsic_host.rs:62`.
   §3.10's rider table settles the amendment question in the opposite direction from the row's
   option (b): row 11, "RNG draw", is graded "**Not a rider.** §2.8 already sanctions it as a
   kernel intrinsic; the key convention is below" (`bsl-language.rst:3310-3314`), and the draft
   ruling that follows fixes the carrier key `(session, tick, domain, stable_key)` and states
   "**The signature stays Phase-2 work** (§2.7)" (`:3393-3399`). The kernel half is already
   landed: `KernelRng::for_carrier(session_id, tick, domain, stable_key)` over a pinned
   `ChaCha8Rng` (`rust/crates/babylon-kernel/src/rng.rs:69-78`), `seed_for`'s length-framed
   SHA-256 (`:53-63`), bit-deterministic `next_f64` (`:92-95`) — which this inventory read and
   cites accurately in §1/§4, then contradicts in §6. The correct blocker statement: a name in
   `DECLARABLE_INTRINSICS` (`declarations.rs:110`, today `["exp","log","floor"]`) and an arm in
   `KernelIntrinsicHost::call` (`intrinsic_host.rs:59-70`, today `floor` alone). For contrast,
   the language *does* contain a genuine categorical prohibition and RNG is not it:
   `PROHIBITED_INTRINSIC_NAMES: [&str; 1] = ["sigmoid"]` (`declarations.rs:116`).

2. **CORRECTION — peripheral revolt is NOT expressible within the landed Slice-1 surface;
   `remove-edge` is refused at content load.** §6 grades this row "PORTABLE WITH D-RECORD,
   favorably reformulable" and proposes
   `(for-each (neighbors periphery EdgeType/EXPLOITATION :out SOCIAL_CLASS) (remove-edge …))`,
   reasoning from `remove-edge`'s presence in `EFFECT_POSITION_ONLY` (`evaluator.rs:464-484`).
   That table records where a verb is *legal by grammar*, not whether a rule using it loads.
   `DEFERRED_SHAPE_VERBS = ["add-node","remove-node","add-edge","remove-edge","add-hyperedge",
   "remove-hyperedge"]` (`structural_verbs.rs:1352-1359`) is refused **at content load** by
   `check_no_deferred_shape_verbs` (`:1388-1406`), which walks the entire rule form — `<when>`
   and `<effects>`, guard/for-each nesting included — and is wired unconditionally into
   `rule_pipeline::load_rule_form` at `rule_pipeline.rs:268`
   (`check_no_deferred_shape_verbs(&rule).map_err(LoadError::DeferredShapeVerb)?`). The reason is
   Task 12's collect-then-apply pre-state split: it "does not serve the graph-shape verbs, only
   update-node/emit/guard/for-each," because "deferring a MINTING verb needs a placeholder-id
   scheme this repair does not specify" (`structural_verbs.rs:691-694`, `:1393-1400`). A rule
   containing `remove-edge` anywhere **does not load at all** on current dev. This row must be
   regraded **BLOCKED — the Task-12 deferred-shape-verb load gate**, and it is the one row the
   inventory graded portable across its entire blocked half.

3. **CORRECTION — Computation 0's "provably `1.0` on every canonical scenario" does not hold; the
   `:const 1.0c` D-record is invalid.** §2 Computation 0 and §6's first row both quote
   `struggle.py:306-308`'s comment ("the six party-less qa scenarios are byte-identical: no
   register ⟹ the multiplier is exactly 1.0") and assert zero party-bearing canonical factories.
   The registry has **twelve** scenarios (`tools/regression_scenarios.py:38-128`), five of which
   are the P25 electoral goldens whose entire purpose is government formation:
   `create_mitterrand_scenario`/`syriza`/`weimar`/`debs`/`bernie_valve`
   (`src/babylon/engine/scenarios/electoral_goldens.py:209,253,310,410,481`), each layering
   duopoly machines, currents and a finance donor via `apply_political_terrain`
   (`electoral_fixture.py:113-204`). `ElectoralSystem` @17.45 writes the register at
   `electoral.py:885` and `:1005` (`graph.set_graph_attr(ELECTORAL_GOVERNMENTS_ATTR,
   governments)`). So on five canonical scenarios a government **is** seated, the guard at
   `struggle.py:254` passes, and `_mean_legitimation`/`compute_legitimation_amplifier` run for
   real. The source comment is stale; the inventory inherited it. Regrade Computation 0 as **live
   on five canonical scenarios**, and treat the `the`-accessor/carrier-`NodeType` graph-scope gap
   as a real blocker there rather than a WS1 filing.

4. **CORRECTION (consequential, to the verdict line) — name both gates.** The verdict line reads
   "(RNG-as-BSL-content-primitive, categorically prohibited; and the `update-edge` substrate-
   storage gap)". Per corrections 1 and 2 the accurate pair is: **(i) the unbuilt RNG intrinsic
   binding** — kernel landed, key convention ruled, signature Phase-2 — and **(ii) the edge lane,
   which is two independent gates, not one**: the `update-edge` substrate-storage refusal
   (`structural_verbs.rs:709-710`, `substrate.rs`'s `add_edge`/`remove_edge`-only trait surface)
   *plus* the Task-12 deferred-shape-verb load gate (`rule_pipeline.rs:268`) that also takes out
   `remove-edge` and `add-edge`. And the closing clause "neither is 'coming soon' the way Slice 1's
   query lane was" is only half right: the RNG binding has a fixed key convention, a landed
   kernel implementation and a ruled conformance methodology, which is a materially shorter
   runway than the substrate-widening decision.

5. **CONFIRMATION — the discarded-backfire defect is real, and it is the best new finding in this
   batch.** Verified line by line. `_update_agitation(node_data, delta)` **returns a new dict**
   and does not mutate its argument (`struggle.py:144-173`: `return {…}` in both branches, no
   in-place assignment to `node_data`). At `:363` the call is `_update_agitation(attrs, backfire)`
   with the return value discarded, and `:364`'s `agitation = _get_agitation_from_node(attrs)  #
   Re-read after update` therefore re-reads the **unchanged** value, which then feeds the
   uprising condition at `:369-371`. No `graph.update_node` carries the backfire either. So
   `consciousness.repression_backfire` and the entire `backfire_multiplier` chain are dead on the
   agitation channel, exactly as reported — and note this compounds with correction 3: the
   legitimation amplifier is *live* on five scenarios and its only consumer is this dead line.
   Transcribe verbatim, D-record, do not fix.

6. **CONFIRMATION — the spontaneous-riot dormancy.** `rg -n "LUMPENPROLETARIAT"
   src/babylon/engine/scenarios/*.py` returns **zero hits** across the whole factory tree; the
   source's own self-declaration at `struggle.py:473` is accurate.

7. **CONFIRMATION — the out-of-`[0,1]` coefficient hazard.** `revolutionary_agitation_boost` is
   `Field(default=0.5, ge=0.0, le=2.0)` (`src/babylon/config/defines/survival.py:230-235`) — the
   same D-1-class shape as Territory's `rent_spike_multiplier`, and correctly graded, since a
   `:field`-sourced operand evaluates as `Value::Real` and the #500 scale op serves
   `Value::Currency` only.

8. **CONFIRMATION — the edge-lane anchors and the `exp` gap.** `update-edge`/`update-hyperedge`
   are grammar-recognised and hard-refused with the reason named (`structural_verbs.rs:709-710`,
   module doc `:16-26`). `field-of` over an `EdgeRef` is unreachable (`evaluator.rs:1190-1192`).
   `KernelIntrinsicHost` dispatches `floor` alone (`intrinsic_host.rs:59-70`, test at `:269-277`)
   — matching the sibling Survival inventory, as claimed. `EFFECT_POSITION_ONLY` is the 19-verb
   table at `evaluator.rs:464-484`, as cited (see correction 2 for what it does *not* establish).

9. **CONFIRMATION — tick position, the 7-system SOLIDARITY channel, and the RESERVED-LINE flag.**
   `position: ClassVar[float] = 16.0` (`struggle.py:235`), between `SurvivalSystem` (15.0,
   `survival.py:78`) and `ConsciousnessSystem` (17.0) in `_SYSTEM_CLASSES`
   (`simulation_engine.py:328-364`); `FascistFactionSystem` at 17.4 lives in `reactionary.py:74,78`
   and is registered — so §5's downstream `ideology` reader list is right. The George Jackson
   bifurcation is correctly flagged RESERVED-LINE (comprador insolvency → labor-aristocracy /
   periphery fork, the `national_identity` axis, the `SocialRole` taxonomy) and described rather
   than proposed upon, per mandate. **One refinement to §5:** two canonical scenarios now seed a
   non-zero `solidarity_strength` — `debs` (`electoral_goldens.py:474`) and `bernie_valve`
   (`:534`), both `0.4` — so the estate-wide "`solidarity_strength=0.0` everywhere" premise
   (`tools/regression_scenarios.py:2833-2836`) is stale, and the 7-system channel this inventory
   identifies is carrying real values on the byte gate today.

**FINAL VERDICT: BLOCKED — sustained, with both blockers restated.** The two gates are (i) the
**unbuilt RNG intrinsic binding** — sanctioned by §2.8, ruled "not a rider" by §3.10 row 11, key
convention fixed, `KernelRng` landed at `babylon-kernel/src/rng.rs`, only the signature and a
`KernelIntrinsicHost` arm missing (NOT a categorical prohibition), and whose conformance target is
ruled to be ensemble-envelope comparison, never byte replay (`rng.rs:30-33`, R8); and (ii) the
**edge lane's two independent gates** — `update-edge`'s substrate-storage refusal and the Task-12
deferred-shape-verb **load** gate (`rule_pipeline.rs:268`), which also refuses `remove-edge`. The
deterministic branches remain portable in their *arithmetic*, but the portable set shrinks:
spontaneous riot stays PORTABLE NOW (modulo the `role`-enum D-record, whose `OrgKind` precedent is
correctly identified); the Jackson bifurcation core stays PORTABLE WITH D-RECORD; **peripheral
revolt is regraded BLOCKED** (correction 2); and **the legitimation backfire multiplier is
regraded live-and-blocked rather than provably-uniform** (correction 3). The title Spark→Uprising
mechanic and every SOLIDARITY-edge strength write remain un-landable as pure BSL content.

**INADEQUATE-COVERAGE NOTE.** A re-read must (i) re-derive every dormancy claim over the
twelve-key `SCENARIOS` registry instead of quoting the source comments' stale "six"/"five" counts
— specifically Computation 0 against the five electoral goldens; (ii) apply
`check_no_deferred_shape_verbs` as a first-class blocker class to every row proposing a graph-shape
verb, and re-audit §6 for any other row reasoning from `EFFECT_POSITION_ONLY` membership alone;
(iii) reconcile §1's accurate reading of `rng.rs` and `determinism-contract.rst` with §6's
contrary "categorically prohibited" grading, which the report currently contains in both
directions.
