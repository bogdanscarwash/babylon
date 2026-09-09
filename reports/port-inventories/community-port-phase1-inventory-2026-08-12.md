# CommunitySystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `CommunitySystem` (`src/babylon/engine/systems/community.py`, 675 lines,
tick position 6.0) computes per-tick ternary consciousness, solidarity amplification, threat
scoring, and alpha-smoothed decay across 14 hardcoded `CommunityType`s, using a third-party XGI
hypergraph rebuilt from scratch every tick as disposable scratch storage. Its state lives almost
entirely **outside the graph substrate** — a plain Python dict inside the optional
`services.community_hypergraph` field, never round-tripped through `WorldState`, and — confirmed
three independent ways (a declared coverage-gap row, a fixture-harvester's own docstring, and a
`STRUCTURALLY_IMPOSSIBLE` seam-registry entry) — **never wired by any of the 12 canonical
`qa:regression` scenarios**, so the system is a structural no-op on the entire current estate, not
merely dormant on part of it. Every computation on the system's primary data path needs a BSL lane
that is explicitly not built today: the hyperedge/metric lane (Slice 3) for community/membership
data, the attributed-membership storage lane (Slice 4 — Director-escalation-gated, and this system
is the ADR189-named first consumer), and the dyadic edge-attribute lane (Slice 2) for
`solidarity_strength`, whose write side (`update-edge`) additionally refuses today because
`GraphSubstrate` models an edge as one bare `f64`. One confirmed libm hazard (`math.log` inside
Shannon-entropy contestation) and one confirmed `Coefficient`-domain overflow defect (solidarity
amplification is unbounded above but writes into a `[0,1]`-declared field) are both port-as-is
transcribable, not blockers in themselves. Zero events, zero Real→Int demotions, zero `Currency`
exposure — a narrower float-hazard surface than Territory's, but a strictly harder architectural
gap.

**Verdict: BLOCKED — needs the hyperedge/metric lane (Slice 3), the attributed-membership storage
lane (Slice 4), and the dyadic edge-attribute lane (Slice 2) before any computation on the
system's primary path is portable; only membership-independent pieces (the substrate-floor
lookup, the Shannon-entropy constant, the class-pair solidarity matrix) are portable today, each
with a D-record — unlike Territory, zero of this system's computations are portable now without
at least Slice 2.**

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/community.py` | 675 | **The target.** `CommunitySystem` (class body 309-370, `step()` 325-370) plus eleven module-level helper functions. Read completely, line by line. Only five module functions are on `step()`'s reachable call graph: `build_community_hypergraph` (57-110), `shared_communities` (113-132), `_extract_memberships_from_node` (282-293), `_get_community_states_from_services` (296-306), and the six private `_`-prefixed step-helpers (373-675). **Six more module-level functions are defined here but never called by `step()` or by any other production module** (grep-confirmed): `communities_spanning_axis` (135-179), `community_overlap_matrix` (182-207), `legal_status_escalate` (210-227), `designate_community` (230-245), `infiltrate_community` (248-262), `disrupt_infrastructure` (265-279) — dead w.r.t. this port's execution scope, exercised only by direct unit-test calls (`tests/unit/engine/systems/test_community_system.py`'s `TestRepressionActions`/`TestCommunitiesSpanningAxis`/`TestCommunityOverlapMatrix`). |
| `src/babylon/formulas/community.py` | 175 | Formula source. `calculate_threat_score` (52-74), `calculate_infrastructure_decay` (77-108), `calculate_solidarity_amplification` (111-141), `compute_community_cost_modifier` (144-174) are all called from `step()`'s reachable path. `calculate_solidarity_potential` (16-49) is registered in the formula registry (`formula_registry.py:124`) but **never called anywhere in `step()`'s reachable path** (grep-confirmed) — dead. |
| `src/babylon/formulas/consciousness.py` | 112 | `compute_ternary_consciousness` (29-109) — called from `_compute_consciousness_from_orgs`. |
| `src/babylon/models/entities/community.py` | 418 | `CommunityState` (249-348), `CommunityMembership` (370-417) Pydantic models. Hardcoded module-level lookup tables: `ROLE_STRENGTH_WEIGHTS` (25-31), `LEGAL_STATUS_MULTIPLIERS` (34-40), `LEGAL_STATUS_ORDER` (43-49, dead re: `step()`), `COMMUNITY_CATEGORY_MAP` (56-75), `HEGEMONIC_COMMUNITIES`/`MARGINALIZED_COMMUNITIES` (83-99), `CONSCIOUSNESS_DEFAULTS` (160-235, dead re: `step()` — scenario-seeding constant, not read by the engine system). **None of these tables are in `defines.yaml`** (see §2e). |
| `src/babylon/models/entities/consciousness.py` | 464 | `TernaryConsciousness` (51-222, the r/l/f 2-simplex + Shannon-entropy `ideological_contestation`), `OrgContribution` (330-353), `SUBSTRATE_FLOOR_DEFAULTS` (356-455, 14-entry hardcoded table, also not in `defines.yaml`), `SubstrateFloor` (294-327). |
| `src/babylon/models/entities/social_class.py` | (relevant: 380, 438-446) | `SocialClass` entity — declares `active: bool` (380), `community_memberships: list[Any]` (438-441), `community_cost_modifier: float, ge=0.0` (442-446, default 1.0). `threat_score` is **not** a declared field here. |
| `src/babylon/models/entities/organization.py` | (relevant: 156-172) | `Organization` entity — declares `cadre_level: Probability` (160), `cohesion: Probability` (156), `consciousness_tendency: ConsciousnessTendency` (172), all read by `_compute_consciousness_from_orgs`. |
| `src/babylon/models/entities/relationship.py` | (relevant: 116-118) | `Relationship` (edge) entity — declares `solidarity_strength: Coefficient` (116-118). `class_pair_solidarity` (written by CommunitySystem) has **no declared field here** — a defect, see §4/§6. |
| `src/babylon/models/enums/community.py` | 85 | `CommunityType` (12-55, **14 members**), `HyperedgeCategory` (58-78, 4 members). |
| `src/babylon/models/enums/social.py` | 212 | `SocialRole` (12-64, 8 members), `MembershipRole` (67-85, 5 members). |
| `src/babylon/models/enums/legal.py` | (relevant: 64-84) | `LegalStatus` (5 members). |
| `src/babylon/models/enums/consciousness.py` | (relevant: 68-84) | `ConsciousnessTendency` (3 members). |
| `src/babylon/models/enums/topology.py` | (relevant: 62-63, 100, 109) | `NodeType.SOCIAL_CLASS`/`.ORGANIZATION`; `EdgeType.SOLIDARITY`/`.MEMBERSHIP`. |
| `src/babylon/config/defines/organizations.py` | (relevant: 12-60) | `CommunityDefines` Pydantic model — `heat_decay_alpha`, `cohesion_decay_alpha`, `infrastructure_decay_alpha`, `community_overlap_bonus` (dead re: `step()`), `rent_differential_penalty` (dead re: `step()`), `core_organizer_maintenance_factor`. |
| `src/babylon/config/defines/economy_class.py` | (relevant: 201-315) | `ClassSystemDefines` — `base_class_solidarity` 5×5 matrix (249-282) + `get_base_solidarity()` lookup (297-315). |
| `src/babylon/config/defines/consciousness.py` | (relevant: 138-143) | `ConsciousnessDefines.education_pressure_decay`. |
| `src/babylon/config/defines/_assembler.py` | (relevant: 165, 185, 203) | `GameDefines` composition — `.consciousness`, `.community`, `.class_system` fields. |
| `src/babylon/data/defines.yaml` | `community:` 447-453; `consciousness:` 210-229; `class_system:` 726-750 | Player-editable coefficient source for the three sub-models above. **Note:** `ROLE_STRENGTH_WEIGHTS`/`LEGAL_STATUS_MULTIPLIERS`/`SUBSTRATE_FLOOR_DEFAULTS`/`_ROLE_TO_CLASS_POSITION` are hardcoded Python dicts, **not** here (§2e). |
| `src/babylon/engine/formula_registry.py` | (relevant: 124-127) | Registers `threat_score`, `solidarity_amplification` (+ `solidarity_potential`, dead) into `services.formulas`. |
| `src/babylon/engine/services.py` | (relevant: 250, 326, 366, 438) | `ServiceContainer.community_hypergraph: Any = field(default=None)` — the **entire system's activation gate**; optional, defaults to `None`, and per §5 is never wired by any production scenario. |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase` — CommunitySystem inherits it but calls **none** of its helper methods (grep-confirmed zero `self.` references in `community.py` — no `_write_clamped`, no `_publish`, no `_read`). |
| `src/babylon/kernel/graph_protocol.py` | (relevant: 77, 88, 152, 258, 278) | `GraphProtocol.get_node`/`.update_node`/`.update_edge`/`.query_nodes`/`.query_edges` signatures. |
| `src/babylon/topology/graph.py` | (relevant: 660-670, 690-702) | Concrete `BabylonGraph.update_node`/`.update_edge` — plain dict merge, **no type coercion or quantization at tick time** (same pattern the Territory inventory established). |
| `src/babylon/kernel/tick_partition.py` | (relevant: 28) | `TickPartition.MATERIAL_BASE`. |
| `src/babylon/engine/simulation_engine.py` | (relevant: 328-364) | `_SYSTEM_CLASSES` tuple — confirms tick position 6.0, between `ReserveArmySystem` (5.0) and `LifecycleSystem` (7.0). |
| `xgi` (third-party PyPI library) | — | `import xgi` (community.py:19). Hypergraph construction (`build_community_hypergraph`) + membership-intersection query (`shared_communities` via `H.nodes.memberships()`). Only these two entry points are on `step()`'s reachable path; `community_overlap_matrix`'s `xgi.incidence_matrix` call is dead code. **No Rust/BSL analog** — the exposed hypergraph model in babylon-graph (Amendment D/AG) is a different, native representation; XGI is scratch-built fresh every tick, never persisted, purely to answer "which community types do these two agents share." |

**Not exercised by `community.py`'s `step()` at all:** no `src/babylon/domain/*` module (grep-confirmed).

**Parallel consumers of the same (dormant) input, outside this port's scope but relevant to §5:**
`src/babylon/projection/community.py`, `src/babylon/projection/topology/levi.py`, and
`src/babylon/ooda/initiative.py` (`compute_community_embeddedness`) all independently read
`SocialClass.community_memberships` — the same field `CommunitySystem` itself reads.

## 2. COMPUTATION CATALOG (execution order, `step()`, community.py:325-370)

### 0. Activation guards (community.py:332-338)
- **(a)** The entire system is a structural no-op unless BOTH a `community_states` config is
  externally wired into the `ServiceContainer` AND at least one `SOCIAL_CLASS` node carries a
  non-empty `community_memberships` list.
- **(b)** `if not community_states: return` (333-334); `if not all_memberships: return` (337-338).
- **(c) Reads:** `services.community_hypergraph` (via `_get_community_states_from_services`,
  296-306); every active `SOCIAL_CLASS` node's `community_memberships` attribute (via
  `_collect_memberships`, 465-479, which also reads `active`, default `True`, line 473).
- **(d) Writes:** none (early-return path).
- **(e) Defines:** none.
- **(f) Events:** none.

Per §5, this guard trips on **every** canonical `qa:regression` scenario today — the whole system
is dormant on the current estate.

### 1. Ternary consciousness from org landscape (`_compute_consciousness_from_orgs`, community.py:373-462)
- **(a)** For each community with members, look at every `ORGANIZATION` node, measure what
  fraction of the community's members belong to that org (density), weight the org's pull toward
  revolutionary/liberal/fascist by its cadre quality and cohesion, and derive a normalized 3-way
  consciousness split for the community — with a floor under the revolutionary component that
  state repression can never fully erase.
- **(b)** Density: `density = overlap / comm_size` (community.py:441), `overlap = len(org_members &
  agents_in_comm)` (438). Per-org weight (consciousness.py:63): `weight = membership_density *
  cadre_level * cohesion`. Tendency accumulation (consciousness.py:67-72): `r_raw += weight` /
  `l_raw += weight` / `f_raw += weight`, branched on `org.tendency`. Unorganized-defaults-liberal
  (consciousness.py:78-79): `unorganized = max(0.0, 1.0 - total_density); l_raw += unorganized`.
  Normalize (consciousness.py:82-95): `total = r_raw+l_raw+f_raw`; if `total < 1e-10`, degenerate
  branch sets `r_norm=substrate_floor, l_norm=1.0-substrate_floor, f_norm=0.0`; else
  `r_norm=r_raw/total` etc. Substrate-floor enforcement (consciousness.py:98-107): if `r_norm <
  substrate_floor`, redistribute `remaining=1.0-substrate_floor` across l/f proportionally (or all
  to l if both ≈0), then `r_norm=substrate_floor`.
- **(c) Reads:** `agent_memberships`; every `ORGANIZATION` node's `consciousness_tendency`/
  `cadre_level`/`cohesion` (community.py:405-414, capped `max_orgs=500`, lines 401, 425-426);
  `MEMBERSHIP` edges from each org (`graph.query_edges(source_id=node.id,
  edge_type=EdgeType.MEMBERSHIP)`, 418-419); `SUBSTRATE_FLOOR_DEFAULTS`
  (consciousness.py:356-455, keyed by `CommunityType`).
- **(d) Writes:** `community_states[comm_type]` in the **services-container dict**, not the
  graph — `state.model_copy(update={"consciousness": new_consciousness})` (460-462). Only
  communities with a nonzero `org_landscape` are recomputed; otherwise the prior tick's
  consciousness is kept verbatim (line 452, `if org_landscape:`).
- **(e) Defines:** none from `GameDefines`/`defines.yaml`. The only coefficient consumed is
  `SUBSTRATE_FLOOR_DEFAULTS[comm_type].floor_value` (`Probability` `[0,1]` per entry — e.g.
  `NEW_AFRIKAN`=0.12, `FIRST_NATIONS`=0.12, `INCARCERATED`=0.18, `CHICANO`=0.08, `WOMEN`=0.04,
  `TRANS`=0.06, `DISABLED`=0.03, `QUEER`=0.04, `UNDOCUMENTED`=0.10, `SETTLER`=0.0,
  `PATRIARCHAL`=0.0, `YOUTH`=0.0, `ADULT`=0.0, `ELDER`=0.02 — consciousness.py:356-455) — a
  **hardcoded module constant, not a `defines.yaml` entry. RESERVED-LINE**: National Question
  consciousness-floor calibration by oppressed nationality, cited to Vera incarceration data /
  Chetty mobility atlas — describe, do not propose changes.
- **(f) Events:** none.

### 2. Hypergraph construction (`build_community_hypergraph`, community.py:57-110)
- **(a)** Rebuilds a fresh XGI hypergraph from scratch every tick: one hyperedge per community
  type with at least one member, carrying the community's full state as hyperedge attributes.
- **(b)** `H.add_edge(members, idx=comm_type.value, heat=float(state.heat),
  cohesion=float(state.cohesion), infrastructure=float(state.infrastructure),
  visibility=float(state.visibility), legal_status=state.legal_status.value,
  reproduction_cost_modifier=state.reproduction_cost_modifier,
  rent_access_modifier=float(state.rent_access_modifier), category=state.category.value,
  consciousness_ci=float(state.consciousness.collective_identity),
  consciousness_tendency=state.consciousness.dominant_tendency.value,
  consciousness_contestation=float(state.consciousness.ideological_contestation))`
  (community.py:93-108). `collective_identity` is a direct passthrough of `r`
  (consciousness.py:157-165). `ideological_contestation` (consciousness.py:194-207) computes
  Shannon entropy via `math.log` when `contestation_stored is None` — **which it always is** for
  every state produced by `compute_ternary_consciousness` (§2.1) or by `CONSCIOUSNESS_DEFAULTS`
  (community.py:160-235, also always native-constructed) — **the one libm call this system's
  reachable path exercises** (consciousness.py:289-291: `entropy -= p * math.log(p)` per
  component, then `entropy / math.log(3)`).
- **(c) Reads:** `all_memberships`; `community_states` dict (for the states referenced by any
  community with ≥1 member).
- **(d) Writes:** none to the graph or services — the XGI `Hypergraph` object is a pure scratch
  structure, discarded when `step()` returns.
- **(e) Defines:** none.
- **(f) Events:** none.

### 3. Solidarity amplification (`_amplify_solidarity_edges`, community.py:527-576)
- **(a)** For every `SOLIDARITY` edge between two agents who share at least one community, boost
  the edge's strength in proportion to how well-organized (infrastructure × cohesion) their shared
  communities are and how committed each agent is to them (membership strength); the base strength
  itself comes from a fixed 5-class solidarity matrix, not the edge's own prior value alone.
- **(b)** `shared = shared_communities(hypergraph, edge.source_id, edge.target_id)` (553) —
  `H.nodes.memberships(agent_a) & H.nodes.memberships(agent_b)` (130-132). `class_pair_solidarity
  = class_system_defines.get_base_solidarity(src_class, tgt_class)` (560, economy_class.py:297-315,
  symmetric dict-of-dict lookup, `0.0` default for unknown pairs). `base_strength =
  edge.attributes.get("solidarity_strength", class_pair_solidarity)` (564) — reads the edge's own
  prior amplified value if present, else falls back to the fresh class-pair lookup (so
  amplification compounds tick-over-tick once the edge already carries a value).
  `_build_shared_data` (482-504) builds per-shared-community `(infrastructure, cohesion, str_a,
  str_b)` tuples, where `str_a`/`str_b` are the two agents' own `CommunityMembership.strength` for
  that community (`0.0` if absent). `amplified = calculate_amplification(base_strength=...,
  shared_communities=shared_data)` → `calculate_solidarity_amplification`
  (formulas/community.py:111-141): `amplification = sum(infra*cohesion*str_a*str_b for ...)`
  (138-140); `return base_strength * (1.0 + amplification)` (141) — **no upper bound**; with
  multiple richly-organized shared communities, `amplification` can exceed `1.0`, so `amplified`
  can exceed the `Coefficient` `[0,1]` domain the field it is written into declares (see §4 defect).
- **(c) Reads:** `agent_memberships` for both edge endpoints (`continue` if either is empty,
  550-551); shared communities from the hypergraph; every `SOCIAL_CLASS` node's `role` (via
  `_get_class_position_name`, 507-524, mapping 8 `SocialRole` values → 5 class-position names via
  the hardcoded `_ROLE_TO_CLASS_POSITION` dict, community.py:43-52, default `"PROLETARIAT"` if role
  is missing/unmapped — no `SocialRole` value is actually unmapped, all 8 are covered); the 5×5
  `class_system_defines.base_class_solidarity` matrix (economy_class.py:249-282, 15 unique values,
  validated `[0,1]` at model-construction time, `_validate_solidarity_matrix`, 284-295); every
  `SOLIDARITY` edge's existing `solidarity_strength` attribute.
- **(d) Writes:** `graph.update_edge(source, target, EdgeType.SOLIDARITY,
  solidarity_strength=amplified, class_pair_solidarity=class_pair_solidarity)` (570-576).
  `class_pair_solidarity` is **not** a declared `Relationship` field
  (`models/entities/relationship.py` has no such field) and is **not** one of the edge attributes
  `WorldState`'s graph-round-trip reconstruction preserves (`world_state.py:360-393` lists exactly
  which attributes survive `from_graph()`; `class_pair_solidarity` is absent) — **written every
  tick, silently dropped on any `WorldState` round-trip**, the same documented "graph round-trip
  loses data" class of gotcha the project already tracks elsewhere.
- **(e) Defines:** none directly (the matrix is a `ClassSystemDefines` field, `defines.yaml:726-750`,
  consumed via the `get_base_solidarity` method, not a scalar coefficient read).
- **(f) Events:** none.

### 4. Threat score (`_compute_threat_scores`, community.py:579-608)
- **(a)** Sum, across all of an agent's community memberships, how much state attention each one
  draws — a product of the community's heat, how visible the agent is in it, how central their
  role is, and how criminalized the community's legal status is.
- **(b)** `graph.update_node(node_id, threat_score=0.0)` for agents with no memberships (590 —
  literal zero, never computed). Else per membership: `(heat, effective_visibility, role_weight,
  legal_mult)` tuples (598-605) — `effective_visibility` is `1.0 if overt else visibility`
  (models/entities/community.py:407-417); `ROLE_STRENGTH_WEIGHTS.get(mem.role, 0.4)` (602,
  hardcoded 5-entry table, community.py:25-31, exhaustive — the `0.4` fallback is dead-but-defensive);
  `LEGAL_STATUS_MULTIPLIERS.get(comm_state.legal_status, 0.1)` (603, hardcoded 5-entry table,
  community.py:34-40, also exhaustive). `calculate_threat_score` (formulas/community.py:52-74):
  `total += heat * visibility * role_weight * legal_mult` (73) — quadruple-multiply accumulate; all
  four factors non-negative so the sum is monotone non-negative but **has no declared upper
  bound** — a single membership in a `CRIMINALIZED`, fully-visible, `CORE_ORGANIZER`, fully-hot
  community caps at `1×1×1×3=3.0`; N memberships sum linearly.
- **(c) Reads:** `agent_memberships`; `community_states` (`heat`, `legal_status` per membership's
  `community_type`).
- **(d) Writes:** `graph.update_node(node_id, threat_score=...)` for every agent (588-608).
  `threat_score` is **not** a declared `SocialClass` model field — it is registered as a graph-only
  computed attribute, exempted from the vocabulary sentinel
  (`sentinels/vocabulary/registry.py:207`) and explicitly excluded from `WorldState.from_graph()`
  reconstruction (`world_state.py:75-78`, `SOCIAL_CLASS_COMPUTED_FIELDS`) — recomputed fresh every
  tick from `community_memberships`, so the exclusion is benign only as long as the source list
  itself survives round-trip (it is a declared field, so it does).
- **(e) Defines:** none from `GameDefines` — `ROLE_STRENGTH_WEIGHTS`/`LEGAL_STATUS_MULTIPLIERS` are
  hardcoded module constants (community.py:25-40), not `defines.yaml` entries.
- **(f) Events:** none.

### 5. Reproduction cost modifier (`_compute_cost_modifiers`, community.py:611-621)
- **(a)** Multiply together every community's reproduction-cost multiplier for each community an
  agent belongs to, producing one compound modifier per agent — intended to feed the cost of
  reproducing labor-power, though nothing downstream actually reads it (see §5).
- **(b)** `compute_community_cost_modifier` (formulas/community.py:144-174): `modifier = 1.0; for
  mem in memberships: state = community_states.get(mem.community_type); if state is not None:
  modifier *= state.reproduction_cost_modifier` (166-173) — pure multiplicative accumulate seeded
  at `1.0`, no clamp, no cap; `reproduction_cost_modifier` (`CommunityState`,
  models/entities/community.py:294-298) is `float, ge=0.0` — **unbounded above** — so the compound
  product is likewise unbounded above (and can shrink toward `0` from any single zero-modifier
  community, since it is a pure product).
- **(c) Reads:** `agent_memberships`; `community_states[mem.community_type].reproduction_cost_modifier`
  for each membership.
- **(d) Writes:** `graph.update_node(node_id, community_cost_modifier=modifier)` (621).
  `community_cost_modifier` **is** a declared `SocialClass` field (social_class.py:442-446, `float,
  ge=0.0`, default `1.0`) — survives round-trip — but **is read by no other system**
  (grep-confirmed against `src/babylon/engine/systems/*.py`, including `lifecycle.py` @7.0, the
  very next system and the one whose reproduction-cost machinery is this value's documented
  intended consumer per its own field description ("Multiplier on V_reproduction for members")): a
  dead write, port-as-is-transcribable defect.
- **(e) Defines:** none directly (reads `community_states`' own field, not a `GameDefines`
  coefficient).
- **(f) Events:** none.

### 6. Community-state decay (`_apply_community_decay`, community.py:624-675)
- **(a)** Every community's heat and cohesion decay geometrically toward zero each tick with no
  counteracting term; infrastructure decays the same way but `CORE_ORGANIZER` members can offset
  the decay via a maintenance term; `education_pressure` (from `EDUCATE`-verb accumulation) also
  decays geometrically.
- **(b)** `new_heat = float(state.heat) * (1.0 - defines.heat_decay_alpha)` (650). `new_cohesion =
  float(state.cohesion) * (1.0 - defines.cohesion_decay_alpha)` (653). `core_count =
  organizer_counts.get(comm_type, 0)` (656, counted by scanning every agent's memberships for
  `mem.role == MembershipRole.CORE_ORGANIZER`, 640-646). `calculate_infrastructure_decay`
  (formulas/community.py:77-108): `maintenance = min(core_organizer_count * maintenance_factor,
  1.0)` (106); `new_value = current * (1.0 - decay_alpha) + maintenance * decay_alpha` (107);
  `return max(0.0, min(1.0, new_value))` (108) — **double clamp**, `max(lo, min(hi, value))` shape,
  textually identical to Territory's `_write_clamped` (`system_base.py:189`). `edu_decay =
  services.defines.consciousness.education_pressure_decay` (665); `new_edu_pressure =
  float(state.education_pressure) * (1.0 - edu_decay)` (666). Final writeback (668-675): `"heat":
  max(0.0, new_heat)` (670), `"cohesion": max(0.0, new_cohesion)` (671), `"infrastructure":
  new_infra` (672, already double-clamped by the formula, no additional clamp here),
  `"education_pressure": max(0.0, new_edu_pressure)` (673) — **three different clamp shapes
  co-exist in one function**: infrastructure gets the full `max(0,min(1,·))` inside the formula;
  heat/cohesion/education_pressure get only a lower-bound `max(0.0, ·)` with **no explicit upper
  clamp at all** (safe here only because decay strictly shrinks an already-≤1 `Probability`-typed
  value scaled by a `[0,1]` retention factor — a property of the caller's data, not enforced by
  this function itself).
- **(c) Reads:** `community_states` (`heat`, `cohesion`, `infrastructure`, `education_pressure`);
  `agent_memberships` (for `CORE_ORGANIZER` counting); `defines.heat_decay_alpha`/
  `.cohesion_decay_alpha`/`.infrastructure_decay_alpha`/`.core_organizer_maintenance_factor`
  (`services.defines.community`, `CommunityDefines`, organizations.py:12-60);
  `services.defines.consciousness.education_pressure_decay` (consciousness.py:138-143).
- **(d) Writes:** `community_states[comm_type] = state.model_copy(update={...})` (668-675) — again
  the **services-container dict**, never the graph.
- **(e) Defines:** `community.heat_decay_alpha` (0.05, `[0,1]`, defines.yaml:448),
  `community.cohesion_decay_alpha` (0.03, `[0,1]`, defines.yaml:449),
  `community.infrastructure_decay_alpha` (0.04, `[0,1]`, defines.yaml:450),
  `community.core_organizer_maintenance_factor` (0.1, `[0,1]`, defines.yaml:453),
  `consciousness.education_pressure_decay` (0.1, `[0,1]`, defines.yaml:224). Two more
  `CommunityDefines` fields (`community_overlap_bonus`=0.1, `rent_differential_penalty`=0.05,
  defines.yaml:451-452) are declared and player-editable but consumed by **nothing** in `step()`'s
  reachable path — they exist only to feed `calculate_solidarity_potential`
  (formulas/community.py:16-49), registered in the formula registry (`formula_registry.py:124`)
  but never called by `CommunitySystem.step()` or any other production code (grep-confirmed) —
  dead defines.
- **(f) Events:** none.

**Events emitted by the whole system: zero.** Grep-confirmed (`community.py`,
`formulas/community.py`, `formulas/consciousness.py`) — no `EventType`/`.publish(`/`emit` anywhere
in the reachable call graph. Same silence as Territory.

## 3. TYPE INVENTORY

**Runtime storage note** (load-bearing, same pattern as Territory's report): `BabylonGraph.update_node`/
`.update_edge` (topology/graph.py:660-670, 690-702) are plain dict merges with no type coercion or
quantization. Pydantic's `SnapToGrid` (1e-5 grid) and range validation apply only when a
`SocialClass`/`Relationship`/`CommunityState` model is *instantiated* (scenario seed, `model_copy`,
or a `WorldState` round-trip) — never mid-tick on a raw graph write. So every in-tick arithmetic
value below is a raw Python float/int with no grid quantization until (if ever) it round-trips
through a model constructor.

**A second, CommunitySystem-specific storage note.** Most of this system's state does **not** live
in the graph at all. `community_states: dict[CommunityType, CommunityState]` lives in
`services.community_hypergraph["community_states"]` — a plain Python dict inside the
`ServiceContainer`, mutated in place via `model_copy`. This is the system's primary state channel
(`heat`, `cohesion`, `infrastructure`, `visibility`, `legal_status`, `consciousness`,
`education_pressure`, `reproduction_cost_modifier`, `rent_access_modifier` — nine of
`CommunityState`'s ten fields) and it is entirely outside `WorldState`/the graph substrate — there
is no `to_graph()`/`from_graph()` round-trip for it at all, and — per the binding disposition —
never will be, because community is never a graph node (Constitution II.7, Anti-Pattern VIII.9,
enforced live by `tests/property/invariants/test_community_membership_lint.py`'s INV-010 linter).
Only two attribute groups actually land on graph nodes/edges: `SOCIAL_CLASS.community_memberships`/
`.community_cost_modifier`/`.threat_score` (node-local) and `SOLIDARITY.solidarity_strength`/
`.class_pair_solidarity` (edge-local).

| Attribute | Node/Edge/Service | Python model type | Domain | Category |
|---|---|---|---|---|
| `community_memberships` | SOCIAL_CLASS (node) | `list[Any]` (a list of `CommunityMembership` records at runtime) | unbounded-length list of structured records | **Unmapped shape — no BSL field type carries a list-of-structs; each element has 5 sub-fields (below)** |
| `community_cost_modifier` | SOCIAL_CLASS (node) | `float, ge=0.0` | `[0.0, ∞)` | unbounded real |
| `threat_score` | SOCIAL_CLASS (node), graph-only | `float` (undeclared model field) | `[0.0, ∞)` in practice (§2.4) | unbounded real, **not a model field** |
| `role` | SOCIAL_CLASS (node) | `SocialRole` (StrEnum, 8 members) | closed set | **Enum discriminant** |
| `active` | SOCIAL_CLASS (node) | `bool` | `{T,F}` | boolean gate |
| `solidarity_strength` | SOLIDARITY (edge) | `Coefficient` (`Annotated[float, ge=0.0, le=1.0]`) | `[0.0, 1.0]` | unit-interval, **but §4 — the write can exceed 1.0** |
| `class_pair_solidarity` | SOLIDARITY (edge) | undeclared (no `Relationship` field) | float, presumably `[0,1]` by construction | **write-only, unmodeled, dropped on round-trip** |
| `consciousness_tendency` | ORGANIZATION (node) | `ConsciousnessTendency` (StrEnum, 3 members) | closed set | **Enum discriminant** |
| `cadre_level` | ORGANIZATION (node) | `Probability` | `[0,1]` | unit-interval |
| `cohesion` | ORGANIZATION (node) | `Probability` | `[0,1]` | unit-interval |
| `CommunityMembership.agent_id` | membership record field | `str` | node-id string | identifier |
| `CommunityMembership.community_type` | membership record field | `CommunityType` (StrEnum, 14 members) | closed set | **Enum discriminant, 14-valued — largest in this estate** |
| `CommunityMembership.role` | membership record field | `MembershipRole` (StrEnum, 5 members) | closed set | **Enum discriminant** |
| `CommunityMembership.strength` | membership record field | `Coefficient` | `[0,1]` | unit-interval |
| `CommunityMembership.visibility` | membership record field | `Probability` | `[0,1]` | unit-interval |
| `CommunityMembership.overt` | membership record field | `bool` | `{T,F}` | boolean |
| `CommunityState.heat` | services dict (not graph) | `Probability` | `[0,1]` | unit-interval |
| `CommunityState.cohesion` | services dict | `Probability` | `[0,1]` | unit-interval |
| `CommunityState.infrastructure` | services dict | `Probability` | `[0,1]` | unit-interval |
| `CommunityState.visibility` | services dict | `Probability` | `[0,1]` | unit-interval (read by hypergraph build only) |
| `CommunityState.legal_status` | services dict | `LegalStatus` (StrEnum, 5 members) | closed set, one-way ratchet (enforced only by the dead `legal_status_escalate`, not by `step()`) | **Enum discriminant** |
| `CommunityState.reproduction_cost_modifier` | services dict | `float, ge=0.0` | `[0,∞)` | unbounded real |
| `CommunityState.rent_access_modifier` | services dict | `Coefficient` | `[0,1]` | unit-interval (read by hypergraph build only, unconsumed further in `step()`'s own reach) |
| `CommunityState.education_pressure` | services dict | `float, ge=0.0` (no upper bound declared) | `[0,∞)` nominally, only ever decayed inside `step()` | unbounded real by declaration |
| `CommunityState.consciousness` (r,l,f) | services dict | `TernaryConsciousness` — 3× `Probability`, simplex-constrained `r+l+f=1.0 ± 1e-4` | 2-simplex | **structured/compound — a 3-vector on a constraint surface, not a scalar** |
| `TernaryConsciousness.contestation_stored` | services dict | `float \| None` | `[0,1]` when set, else `None` (triggers Shannon-entropy computation) | **nullable real — a sentinel-typed field** |
| `heat_decay_alpha`, `cohesion_decay_alpha`, `infrastructure_decay_alpha`, `core_organizer_maintenance_factor` (defines) | — | `float` | `[0,1]` | unit-interval coefficients |
| `education_pressure_decay` (define) | — | `float` | `[0,1]` | unit-interval coefficient |
| `ROLE_STRENGTH_WEIGHTS` values | hardcoded dict, not a define | `float` | `{1.0, 0.7, 0.4, 0.2, 0.1}` | fixed lookup table, **not player-moddable today** |
| `LEGAL_STATUS_MULTIPLIERS` values | hardcoded dict, not a define | `float` | `{0.1, 0.5, 1.0, 2.0, 3.0}` | fixed lookup table, **unbounded above the [0,1] norm** (max 3.0), not player-moddable |
| `SUBSTRATE_FLOOR_DEFAULTS[*].floor_value` | hardcoded dict, not a define | `Probability` | `[0,1]` per entry | RESERVED-LINE (National Question consciousness-floor calibration) |
| `base_class_solidarity` (matrix) | `ClassSystemDefines` field, IS in defines.yaml | `dict[str, dict[str, float]]` | `[0,1]` per cell (validated), 15 unique values | closed 5×5 symmetric matrix |

**Enum discriminant flag — the largest cardinality in the estate.** `CommunityType` is 14-valued
(`models/enums/community.py:12-55`) — the largest single enum this port would need, well past
Territory's 5-valued `TerritoryType`. Per the task brief's CURRENT BSL surface note, enum fields
ARE landed (ADR195/196: `defenum`, `deffield <field> enum <EnumName>`, `=` comparison,
declaration-order-is-ordinal) — so `CommunityType`/`MembershipRole`/`LegalStatus`/
`ConsciousnessTendency`/`SocialRole` are all individually representable as BSL enums now, unlike
Territory's Phase-1 finding. The blocker for this system is not enum *storage* but the **shape**
the enum-valued data is packaged in — a per-node LIST of `(community_type, role, strength,
visibility, overt)` records — which is the attributed-membership problem (§6), not an enum
problem per se.

**Currency flag — does not apply.** No `Currency`-typed field is read or written anywhere in
`CommunitySystem`'s reachable path (grep-confirmed) — this system never touches money-semantic
values, unlike Territory's `rent_level`.

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `float`), except the one confirmed libm hazard below.
Shapes, in execution order:

1. **Division (density):** `overlap / comm_size` (community.py:441) — int/int → float, `comm_size`
   is `len()` of a nonempty set (guarded, line 431), so no division-by-zero.
2. **Triple multiply (org weight):** `membership_density * cadre_level * cohesion`
   (consciousness.py:63).
3. **Accumulate-by-branch:** `r_raw += weight` / `l_raw += weight` / `f_raw += weight`
   (consciousness.py:67-72) — three-way conditional accumulation.
4. **Subtract + clamp:** `unorganized = max(0.0, 1.0 - total_density)` (consciousness.py:78) —
   bare `1.0` literal (BSL "no bare non-integer literal" constraint, same as Territory's finding).
5. **Triple-sum + threshold-branch normalize:** `total = r_raw + l_raw + f_raw`; `if total <
   1e-10:` (consciousness.py:82-83) — bare `1e-10` scientific-notation literal (a second, distinct
   bare-literal shape from item 4's `1.0`).
6. **Three divisions (normalize):** `r_norm = r_raw / total`, `l_norm = l_raw / total`, `f_norm =
   f_raw / total` (consciousness.py:93-95).
7. **Substrate-floor redistribution:** `remaining = 1.0 - substrate_floor` (99); `lf_sum = l_norm +
   f_norm` (100); `if lf_sum > 1e-10:` (101, same bare-literal shape as item 5); `l_norm = l_norm *
   remaining / lf_sum` / `f_norm = f_norm * remaining / lf_sum` (102-103) — multiply-then-divide,
   twice.
8. **`math.log` — THE libm hazard** (consciousness.py:289-291), reachable every tick
   `build_community_hypergraph` runs (community.py:107): `entropy -= p * math.log(p)` for each of
   `(r, l, f)` where `p > 1e-10`; `return entropy / math.log(3)`. Up to three transcendental calls
   per `ideological_contestation` evaluation, plus one fixed `math.log(3)` recomputed every time
   rather than cached. `log` **is** in `DECLARABLE_INTRINSICS` (`declarations.rs:110`, `["exp",
   "log", "floor"]`) — expressible in BSL syntax — but cross-implementation floating-point
   transcendentals do not reproduce bit-for-bit across languages (CLAUDE.md's own tolerance-policy
   discipline); a Rust port needs an explicit tolerance derivation for any conformance check
   against this value, not a byte-identical assertion.
9. **Sum-of-products (amplification):** `sum(infra * cohesion * str_a * str_b for infra, cohesion,
   str_a, str_b in shared_communities)` (formulas/community.py:138-140) — quadruple multiply per
   shared community, summed.
10. **Multiply with bare-literal offset:** `base_strength * (1.0 + amplification)`
    (formulas/community.py:141) — bare `1.0` again, and **no upper clamp at all** — flagged defect
    (§2.3, §6): the `Coefficient` field this feeds declares `[0,1]`, but this formula's output is
    unbounded above.
11. **Quadruple multiply, accumulate (threat score):** `total += heat * visibility * role_weight *
    legal_mult` (formulas/community.py:73) — same "accumulate with no clamp" shape as item 10;
    `legal_mult` alone ranges up to 3.0, so a single membership can score above 1.0, and summing
    several does not saturate.
12. **Multiplicative accumulate (cost modifier):** `modifier *= state.reproduction_cost_modifier`
    (formulas/community.py:173), seeded at `1.0` — a *product*, not a sum; unlike every other
    accumulator in this system, this one can shrink toward 0 as easily as grow.
13. **Decay (heat/cohesion/education_pressure) — 3 occurrences, same shape:** `current * (1.0 -
    alpha)` (community.py:650, 653, 666) — subtract + multiply, bare `1.0` each time.
14. **Infrastructure decay's compound formula:** `maintenance = min(core_organizer_count *
    maintenance_factor, 1.0)` (formulas/community.py:106) — int×float multiply, then an
    upper-only `min` against a bare `1.0`; `new_value = current * (1.0 - decay_alpha) + maintenance
    * decay_alpha` (107) — two multiplies, one subtract, one add; `return max(0.0, min(1.0,
    new_value))` (108) — **the full double-clamp `max(lo, min(hi, ·))` shape**, textually
    identical to Territory's `_write_clamped` (`system_base.py:189`).
15. **Clamps, three DIFFERENT shapes in the same function** (`_apply_community_decay`,
    community.py:668-675):
    - Infrastructure: full double clamp, INSIDE the formula (item 14).
    - Heat/cohesion/education_pressure: lower-only `max(0.0, new_X)` (community.py:670, 671, 673) —
      **no explicit upper clamp at all**, relying on the caller-side invariant that a
      `Probability`-typed starting value times a `[0,1]` retention factor cannot exceed its own
      start (true here, but not enforced by this function — a latent trap if `alpha` were ever
      allowed outside `[0,1]`, which `CommunityDefines`'s `ge=0.0, le=1.0` field constraint
      currently forecloses).
    - (Dead code, for completeness, since the whole file was read): `designate_community`'s
      `new_heat = min(1.0, float(escalated.heat) + heat_increase)` (community.py:244) —
      upper-only, the mirror image of the live code's lower-only shape. Not reachable from `step()`.
16. **No Real→Int demotions anywhere** in the reachable path (grep-confirmed zero `int(...)` casts
    in `community.py`/`formulas/community.py`/`formulas/consciousness.py`) — unlike Territory, this
    system has no truncation/floor hazard at all.
17. **Fixed loop bounds, Power-of-10-compliant:** `max_orgs = 500` appears twice independently —
    once collecting org data (community.py:401, 425-426) and once inside
    `compute_ternary_consciousness` iterating `org_landscape` (consciousness.py:61, 74-75) — two
    separate caps on what is, after the first cap, already a ≤500-element list, so the second cap
    is dead-but-defensive in the current call shape, not a live double-application.

**libm hazard verdict: YES — exactly one hazard site** (`math.log`, item 8), reachable on every
tick the system runs at all (any nonzero `community_states`), unlike Territory's confirmed zero.

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 6.0** (community.py:319), confirmed against `_SYSTEM_CLASSES`
  (`simulation_engine.py:328-364`): `... ReserveArmySystem (5.0) → CommunitySystem (6.0) →
  LifecycleSystem (7.0) → SolidaritySystem (8.0) → ...`.
- **Reads from a same-tick prior system: one confirmed channel.** `VitalitySystem` @1.0's "Reaper"
  phase (vitality.py:158-174) can set `SOCIAL_CLASS.active=False` on extinction/starvation/
  zombie-trap; `_collect_memberships` (community.py:472-474) skips any `SOCIAL_CLASS` node with
  `active` falsy — so a class Vitality kills off this tick is invisible to Community this same
  tick. No other system positioned 1.0-5.0 (Territory, Substrate, Production, TickDynamics,
  ReserveArmy) writes any attribute Community reads (`active`, `community_memberships`, `role`,
  SOLIDARITY/MEMBERSHIP edges) — grep-confirmed against each system's `update_node`/`update_edge`
  call sites.
- **Writes consumed later this tick / downstream ticks:**
  - `SOLIDARITY.solidarity_strength` — **Community's single most load-bearing output**, read/
    written by 8 other systems (grep-confirmed): `solidarity.py` (@8.0, immediately next),
    `survival.py` (@15.0), `struggle.py` (@16.0), `ideology.py`, `doctrine.py` (@14.7),
    `policy.py` (@17.47), `electoral.py` (@17.45), `reactionary.py`. Community's amplification is
    one write among many on this field — every downstream reader treats it as "the" edge strength
    regardless of who wrote it last.
  - `SOCIAL_CLASS.threat_score` — grep-confirmed **read by no other System**
    (`src/babylon/engine/systems/*.py`) — terminal/observational, not a model field (excluded from
    round-trip, §3).
  - `SOCIAL_CLASS.community_cost_modifier` — grep-confirmed **read by no other System**, including
    `lifecycle.py` @7.0, whose reproduction-cost machinery is this value's documented intended
    consumer — a dead write (§2.5, §6).
  - `SOLIDARITY.class_pair_solidarity` — grep-confirmed **read by no other System, and not even a
    declared model field** — write-only, dropped on any `WorldState` round-trip (§2.3, §3).
  - `services.community_hypergraph["community_states"]` (heat/cohesion/infrastructure/
    consciousness/education_pressure/…) — grep-confirmed **read by no other System**
    (`rg 'community_hypergraph' src/babylon/engine/systems/*.py` returns only `community.py`
    itself) — the system's primary state channel is entirely self-contained; nothing else in the
    engine observes it.
- **Context/service usage with no BSL equivalent:**
  - `services.community_hypergraph: Any = field(default=None)` (`engine/services.py:250`) — the
    entire system's activation gate lives in an OPTIONAL, untyped (`Any`) service field, not a
    graph read. BSL's evaluation model has no analog to "a service container field that may or may
    not be populated" — the closest precedent is Territory's `TickContext.displacement_mode`, but
    that was a per-run test override on top of an otherwise-live system; here the override IS the
    only way the system is ever live at all (§6).
  - `services.formulas.get("threat_score")` / `.get("solidarity_amplification")`
    (community.py:542, 586) — a runtime formula-registry indirection; on every real path it
    resolves to the exact functions catalogued in §2, but the indirection itself (swap a formula at
    runtime via `calculator_overrides`) has no BSL equivalent — BSL rules are content, not injected
    callables.
  - `xgi.Hypergraph` (third-party PyPI library, community.py:19) — rebuilt from scratch every
    tick, discarded at tick end, used only to answer "which community types do two agents share"
    (`shared_communities`, community.py:113-132, via `H.nodes.memberships()`). Nothing about XGI's
    own object model needs replicating in a port, only the set-intersection behavior it computes.
- **Dormancy — confirmed TOTAL on the current canonical estate, not partial like Territory's.**
  `services.community_hypergraph` is **never wired by any scenario builder** — confirmed three
  independent ways:
  1. `tools/regression_scenarios.py`'s own declared `COVERAGE_GAPS` table (lines 2848-2855) names
     `CommunitySystem` explicitly: *"services.community_hypergraph is None (the plain in-memory
     step() API's default) and no MEMBERSHIP edges are seeded; the step() body returns on its
     first or second guard clause every tick."*
  2. `tools/record_community_fixture.py`'s module docstring (lines 12-25): *"no scenario builder in
     this codebase ever populates SocialClass.community_memberships ... confirmed absent from
     build_single_county_overrides and every engine/scenarios/*.py builder."*
  3. `src/babylon/sentinels/seam/registry.py:2170-2192` marks the `community_memberships` payload
     `liveness_class=LivenessClass.STRUCTURALLY_IMPOSSIBLE` — the strongest liveness category the
     registry has, reserved for "no runtime condition can light it — only a code/data change."

  `SCENARIOS` (`tools/regression_scenarios.py:38-128`, AST-verified) declares **12 named canonical
  scenarios** (`imperial_circuit`, `two_node`, `starvation`, `glut`, `fascist_bifurcation`,
  `single_county`, `mitterrand`, `syriza`, `weimar`, `debs`, `bernie_valve`, `org_probe`);
  `community_hypergraph`/`community_memberships` appear in **none** of them (grep-confirmed
  against `tools/regression_scenarios.py` and every `src/babylon/engine/scenarios/*.py` builder).
  `EdgeType.MEMBERSHIP` edges ARE seeded, but only by `electoral_goldens.py`/`electoral_fixture.py`
  (electoral-specific fixtures, not the canonical `qa:regression` set) — and even there,
  `community_hypergraph` is still unwired, so `step()` still returns at its FIRST guard
  (`community_states` empty) before ever reaching `_collect_memberships`. **Net effect: the
  `qa:regression` byte-identical gate provides ZERO conformance signal for CommunitySystem
  today** — not "Phase 1 only" like Territory, but nothing at all. A port's conformance fixtures
  must be hand-built from scratch (as `tools/record_community_fixture.py` itself documents doing,
  honestly, for the adjacent projection-layer fixture).
- **A parallel, independent consumer of the same dormant substrate.**
  `babylon.ooda.initiative.compute_community_embeddedness` (`ooda/initiative.py:82-152`, wired live
  into `OODASystem` @14.0 at `ooda.py:155`) reads the SAME `SocialClass.community_memberships`
  field CommunitySystem itself reads — walking org→territory→TENANCY-linked members→membership
  presence — and is explicitly documented as "structurally 0.0 in every real game today"
  (`initiative.py:103-112`) for the identical reason. This is not a channel FROM CommunitySystem
  (it doesn't read anything CommunitySystem writes), but confirms the dormancy is substrate-wide,
  not local to this one system's `step()`.

## 6. BLOCKER ASSESSMENT

Adjudicated against the CURRENT BSL surface stated in the task brief (Query lane Slice 1 landed;
Slices 2-4 not built; enums landed ADR195/196; `exp`/`log`/`floor` declarable; Amendment AG
ratified but its grammar/evaluator support explicitly deferred to a follow-up spec + implementation;
events unpinnable).

| Computation | Verdict | Detail |
|---|---|---|
| Ternary consciousness from org landscape (community.py:373-462) | **BLOCKED — hyperedge/metric lane (Slice 3) + a genuinely unmapped storage shape** | Needs an ORGANIZATION node scan (`nodes` — landed) crossed with MEMBERSHIP-edge target enumeration (`neighbors`-shaped, landed IF only target ids are needed — confirmed true here) — but the community-membership DENSITY computation needs `community_memberships`-shaped per-node list data with no BSL field-type analog at all (not even the attributed-membership kind, which models hyperedge-side membership, not this node-local list). The read is against the wrong element kind for this system's actual production data shape. |
| Substrate-floor lookup (community.py:453-454, `SUBSTRATE_FLOOR_DEFAULTS`) | **PORTABLE WITH D-RECORD** | A closed, 14-entry per-`CommunityType` constant table — expressible as 14 `defconst`s keyed by a `CommunityType` enum member, once `CommunityType` itself is declared (`defenum`, landed). D-record: RESERVED-LINE content (National Question consciousness floors) — the port transcribes the numbers verbatim, no theory judgment made by the port itself. |
| Shannon-entropy contestation (`math.log`, consciousness.py:274-291, reached via community.py:107) | **PORTABLE WITH D-RECORD** | `log` is declarable (`DECLARABLE_INTRINSICS`). D-record: libm cross-implementation tolerance policy required (CLAUDE.md's own standing rule — no byte-identical assertion across a transcendental). Independent of the hyperedge-lane blockers — this hazard fires the moment ANY `CommunityState.consciousness` is read, regardless of how membership data reaches the pack. |
| Hypergraph construction / `shared_communities` (community.py:57-132) | **BLOCKED — hyperedge lane (Slice 3)** | `hyperedges`/`members-of`/`hyperedges-of` are explicitly Slice 3, unserved (`evaluator.rs:507-510`). The XGI object itself is disposable scratch (§5) — a port would use `members-of`/`hyperedges-of` directly against native hyperedges instead of rebuilding a third-party hypergraph object — but that lane is not served yet either way. |
| Solidarity amplification (community.py:527-576) | **BLOCKED — dyadic edge lane (Slice 2) AND an effect-verb storage gap** | Reading `edge.attributes.get("solidarity_strength", ...)` needs `edges`/`edge-between`/`the` (Slice 2, unserved, `evaluator.rs:504-506`). Writing it back needs `update-edge`, which is grammar-recognized (D35) but REFUSES even in effect position today with an explicit storage-gap message: *"GraphSubstrate keys an edge to one f64 strength and gives a hyperedge no attributes at all. Widening that state widens the canonical state_hash field set, which is a declared Phase-2/substrate decision"* (`structural_verbs.rs:374-381`). This system needs to write TWO named edge attributes (`solidarity_strength` and the dead `class_pair_solidarity`) where the current substrate models an edge as a single scalar — a strictly harder mismatch than Territory ever hit (Territory never wrote edge attributes at all). |
| `_get_class_position_name` / base_class_solidarity matrix (community.py:507-524, economy_class.py:297-315) | **PORTABLE WITH D-RECORD** | `role` reads as a landed enum field; the 5×5 matrix is a closed, symmetric, `[0,1]`-validated lookup — expressible as up to 15 `defconst`s (or nested `if`/`=` guards on the two class-position enum values). No hyperedge/edge-attribute dependency in this piece alone. |
| Threat score (community.py:579-608) | **BLOCKED — same membership-shape dependency as row 1, plus hardcoded (non-`defines.yaml`) lookup tables** | Even setting aside the `community_memberships` shape problem, `ROLE_STRENGTH_WEIGHTS`/`LEGAL_STATUS_MULTIPLIERS` are hardcoded Python dicts, not `GameDefines` — portable as `defconst` tables once the enclosing membership-iteration blocker clears, but the port would need to either promote them to `defines.yaml` first (a scope decision for #536, not this inventory) or transcribe them as BSL constants directly. |
| Reproduction cost modifier (community.py:611-621) | **BLOCKED — same membership-shape dependency; also a dead write (§5)** | The formula itself (`compute_community_cost_modifier`) is a trivial multiplicative fold once membership data is reachable — but no BSL lane exists to reach it, and the value is unread downstream in the frozen system anyway (a port-scope question: transcribe the dead write faithfully port-as-is, or flag it a WS1 ledger item — a Director/#536 decision, not this inventory's). |
| Community-state decay (community.py:624-675) | **BLOCKED — architecturally, not computationally** | The arithmetic itself (three geometric-decay shapes + one double-clamp) is the SAME shape class as Territory's Phase-1 heat decay (already ruled PORTABLE NOW there) — no libm, no query-lane dependency, all coefficients are `[0,1]`-domain `defconst`-able. The blocker is that `community_states` is not graph state at all (§3) — it lives in the services container, keyed by `CommunityType`, with no hyperedge/node/edge home in the current OR the AG-ratified model (AG's attributed-membership kind models the *(member, hyperedge)* pair's payload, not the hyperedge's OWN state — heat/cohesion/infrastructure/etc. are community-level, not membership-level). This is a genuinely unnamed gap: even a fully-built hyperedge lane (Slice 3) plus attributed membership (Slice 4) would not, on the spec text read (`bsl-language.rst` §2.9-2.12), obviously give a `CommunityState`-shaped hyperedge its OWN scalar fields the way a node or edge gets `deffield`s — confirm at the #536 design gate, do not assume. |
| Solidarity-strength `Coefficient`-domain overflow (formulas/community.py:141) | **NOT-A-PACK-BLOCKING, but a required port-as-is D-record** | The frozen formula's output is genuinely unbounded above while the field it targets is declared `[0,1]` — a pre-existing defect in the frozen system, not a BSL limitation. Port-as-is law: transcribe the unclamped formula verbatim, D-record the domain mismatch, do not silently add a clamp the frozen system never had. |
| Zero event emissions (whole system) | **N/A — nothing to port** | No `EventType` emission anywhere in the reachable path; the task brief's "every EventType emission is a WS1 ledger row" concern does not apply to this system at all. |

**Summary verdict for #536.** Every computation this system performs on its PRIMARY data path
(community-level state, membership-level payload) is blocked on a real, named, verified BSL
lane — the hyperedge/metric lane (Slice 3), the attributed-membership storage lane (Slice 4,
itself Director-escalation-gated per `evaluator.rs:494-495` — "deferred to first consumer," and
this system IS the named first consumer per ADR189's own text), and the dyadic edge-attribute lane
(Slice 2) for the one piece of state (`solidarity_strength`) that DOES live on an ordinary graph
edge. The only genuinely self-contained, lane-independent pieces are the substrate-floor lookup,
the Shannon-entropy hazard, and the class-position/solidarity-matrix lookup — none of which can
fire without the membership data the blocked lanes are needed to reach in the first place. Unlike
Territory (three of seven computations portable now), **CommunitySystem has zero computations
portable today without at minimum Slice 2, and its primary content needs Slice 3 and Slice 4 as
well** — consistent with the task brief's framing that this is its own chartered train (#536), not
a subset foldable into the query-evaluation unblock that closed Territory's gate.

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_community_system.py` | 540 | **Primary conformance-oracle candidate for `step()`'s own behavior**, plus direct unit coverage of the six module-level functions `step()` never calls (`TestBuildCommunityHypergraph`, `TestSharedCommunities`, `TestCommunityOverlapMatrix`, `TestCommunitySystemStep`, `TestRepressionActions` [covers `legal_status_escalate`/`designate_community`/`infiltrate_community`/`disrupt_infrastructure`, community.py:359-413], `TestCommunitiesSpanningAxis`, `TestBuildCommunityHypergraphUpgrade`). A port targeting `step()` only needs the `TestCommunitySystemStep`/`TestBuildCommunityHypergraph`/`TestSharedCommunities` classes; the rest exercise dead-w.r.t.-`step()` code. |
| `tests/unit/engine/laws/test_law_community_system.py` | 282 | **Property-based behavioral-contract laws** (Hypothesis), already pre-written as exactly the "rewrite test" artifact class CLAUDE.md's testing philosophy calls for: L1 (inactivity, no config), L2 (inactivity, no memberships — TWO independent guard clauses, both must trip), L3 (threat_score non-negativity + the literal-zero no-memberships branch), L4 (heat/cohesion decay never increases; infrastructure stays in [0,1] regardless of organizer count). These four laws map directly onto this inventory's §2 findings and are a ready-made starting oracle independent of bit-exactness. |
| `tests/unit/formulas/test_community_formulas.py` | 275 | Direct unit coverage of all five `formulas/community.py` functions, INCLUDING `calculate_solidarity_potential` (`TestSolidarityPotential`, dead w.r.t. `step()`) and `TestSolidarityPotentialWithMatrix` (Feature 038 class-pair matrix values) — a conformance-oracle candidate for the arithmetic in isolation, decoupled from graph/service plumbing. |
| `tests/unit/models/test_community_models.py` | 743 | `CommunityState`/`CommunityMembership`/`TernaryConsciousness`-adjacent model validation (field bounds, the simplex constraint, category auto-assignment) — schema-level, not tick-behavior; the largest test file in the estate for this system. |
| `tests/property/invariants/test_community_membership_lint.py` | 143 | **INV-010 structural linter** (spec-055 US2): enforces Constitution II.7 / Anti-Pattern VIII.9 — "no MEMBERSHIP edge has a community-node source," i.e., mechanically re-proves the binding disposition this task brief states ("community is NEVER a graph node"). Three predicates: full-pipeline post-state (A), MEMBERSHIP-edge-count-delta legitimacy (B), seeded-violation-is-caught negative test (C). The test file most directly relevant to #536's own chartering constraint — any port content pack must keep passing this invariant's spirit (no community hyperedge ever gains a member via a fabricated pairwise edge). |
| `tests/unit/projection/test_community_fixture_recorder.py` | 110 | Projection/`observe()`-layer fixture recording — not engine math. |
| `tests/unit/projection/test_community.py` | 226 | `babylon.projection.community` view-model tests — narrative/`observe()` layer, not conformance. |
| `tests/unit/projection/vault/test_materializer_community.py` | 53 | Golden-vault materializer test for the community projection page — byte-gate adjacent (III.13 estate) but downstream of engine math, not a substitute for it. |
| `tests/unit/projection/vault/test_render_community.py` | 84 | Vault rendering test, same layer as above. |

**qa:regression byte-gate coverage: NONE (confirmed, not partial).** Unlike Territory (whose
Phase-1 heat dynamics WAS live on canonical scenarios even though Phases 2-4 were dormant),
`tools/regression_test.py::graph_content_hash`'s node/edge-attribute hash technically WOULD catch
a change to `SOCIAL_CLASS.community_cost_modifier`/`.threat_score` or
`SOLIDARITY.solidarity_strength`/`.class_pair_solidarity` IF CommunitySystem ever wrote them on a
canonical scenario — but per §5's three-way-confirmed dormancy finding, it never runs at all on
any of the 12 declared `SCENARIOS`. **The byte-identical gate provides zero live regression
protection for this system today; the `.bscn` conformance fixtures a port would need are unattested
by ANY existing golden and must be built from scratch**, exactly as `tools/record_community_fixture.py`
already had to do (honestly, all-`None`) for the adjacent projection-layer fixture.

---

## Adjudication (2026-08-12)

Adjudicated against the **current dev tree** (`9324482f`) by a second, read-only
pass, in the manner of the Territory inventory's own "Adjudicated verdict"
section. Two corrections, six confirmations. The BLOCKED verdict survives and
hardens; one blocker is mislabelled and one PORTABLE row is not portable.

1. **CORRECTION — the `_get_class_position_name` / solidarity-matrix row is not
   PORTABLE WITH D-RECORD; the `role` half is D102-blocked.** The row's premise
   is "`role` reads as a landed enum field". It reads `role` off **both
   SOLIDARITY endpoints**, not off any rule subject:
   `src/babylon/engine/systems/community.py:507-524` takes a `node_id`, calls
   `graph.get_node(node_id)` and reads `attrs.get("role", "")`, invoked twice at
   `:558-559` for `edge.source_id`/`edge.target_id`. In BSL that is
   `(field-of it social-class/role)` — the landed fold idiom
   (`rust/crates/babylon-tick/tests/query_lane_e2e.rs:152-153, 238-239`) — and
   `field-of` naming an `:enum-type`-declared field is **REFUSED AT LOAD**:
   `rust/crates/babylon-bsl/src/typecheck.rs:266-289`
   (`check_no_field_of_on_enum_field`) — *"field-of {qname}: an
   :enum-type-declared field is read via a :field binding only (§2.5) —
   field-of is not extended to enum-declared fields (§2.13, D102)"*, no error
   code minted (`:804-805` pins that). This contradicts §3's own claim that
   `SocialRole` and the rest "are all individually representable as BSL enums
   now, unlike Territory's Phase-1 finding" — representable as *stored* fields,
   yes; readable off a non-subject node, no. Portable only under an int-ordinal
   `role` encoding, which is a different D-record from the `defenum` one §3
   recommends and is mutually exclusive with it. **The 5×5
   `base_class_solidarity` matrix half of the row stands** — it is a closed
   `[0,1]`-validated table (`src/babylon/config/defines/economy_class.py:249-282`)
   expressible as `defconst`s.
2. **CORRECTION — the executive verdict and row 1 mislabel the
   ternary-consciousness blocker as Slice 3, where the report's own detail
   proves a harder finding.** The verdict line and the row headline read
   "BLOCKED — hyperedge/metric lane (Slice 3) + a genuinely unmapped storage
   shape", but the row's body correctly says "the read is against the wrong
   element kind for this system's actual production data shape". Verified: the
   binding constraint is `SocialClass.community_memberships`, a **node-local
   list of 5-field records** (`src/babylon/models/entities/social_class.py:438-441`,
   `list[Any]`), and no numbered slice serves it — slice 3 mints
   `hyperedges`/`members-of`/`hyperedges-of`/`metric-of`
   (`rust/crates/babylon-bsl/src/evaluator.rs:507-510`), which address a
   hyperedge's members, and slice 4's `membership-field-of` (`:511`) addresses
   the *(member, hyperedge)* pair's payload — neither is a per-node
   list-of-structs field type. The honest label is **BLOCKED — unmapped storage
   shape, served by NO numbered slice**, which is a *stronger* finding than the
   report's own headline: naming a roadmap slice implies a landing date the
   roadmap does not carry. Correct the headline up, not down.
3. **CONFIRMATION — the three-way dormancy proof is the strongest in this
   batch, and every leg was re-verified independently.** (a)
   `tools/regression_scenarios.py:2848-2855` reads verbatim as quoted, naming
   `CommunitySystem` and both guard clauses. (b) `SCENARIOS`
   (`tools/regression_scenarios.py:37-133`) is exactly the 12 keys listed, and
   `create_scenario`'s factory dispatch (`:167-190`) names nine factories; `rg -n
   "community_memberships" src/babylon/engine/scenarios/ tools/` returns only
   `tools/record_community_fixture.py`'s own docstring, never a seeder. (c)
   `src/babylon/sentinels/seam/registry.py:2170-2192` carries
   `liveness_class=LivenessClass.STRUCTURALLY_IMPOSSIBLE` with `write_site`
   *"no scenario builder assigns SocialClass.community_memberships anywhere in
   production"* and the note *"no runtime condition can light it — only a
   code/data change."* **Critically, unlike the ReserveArmySystem inventory's own
   dormancy legs in this same batch, none of these rests on the stale "five
   scenarios" phrasing**, and the `STRUCTURALLY_IMPOSSIBLE` class is
   scenario-count-independent. The step-body guards are confirmed by direct read
   (`community.py:332-338`). Total dormancy: **CONFIRMED.**
4. **CONFIRMATION — the `update-edge` storage refusal is verbatim as quoted;
   cite drift only.** The arm is at
   `rust/crates/babylon-bsl/src/structural_verbs.rs:387-395` (the report cites
   `:374-381`), `verb @ ("update-edge" | "update-hyperedge") => Err(plain(…))`,
   with the message *"GraphSubstrate keys an edge to one f64 strength and gives a
   hyperedge no attributes at all. Widening that state widens the canonical
   state_hash field set, which is a declared Phase-2/substrate decision
   (Constitution III.7)"*; the verb's existence-not-unknown-head framing is
   documented at `:16`, with a second refusal at `:709`. The two-named-attribute
   write (`solidarity_strength` + `class_pair_solidarity`,
   `community.py:570-576`) against a one-`f64` edge model is confirmed as the
   strictly harder mismatch the report claims.
5. **CONFIRMATION — the channel table's most load-bearing claim, spot-checked.**
   `rg -ln "solidarity_strength" src/babylon/engine/systems/` returns nine files:
   `community.py` plus exactly the eight the report names — `solidarity.py`,
   `policy.py`, `struggle.py`, `doctrine.py`, `ideology.py`, `electoral.py`,
   `survival.py`, `reactionary.py`. `rg -ln
   "community_cost_modifier|threat_score" src/babylon/engine/systems/` returns
   `community.py` alone, confirming both dead-write findings (including that
   `lifecycle.py` @7.0, the documented intended consumer, does not read it).
6. **CONFIRMATION — tick position 6.0.** `src/babylon/engine/systems/community.py:319`
   (`position: ClassVar[float] = 6.0`) against the 34-member `_SYSTEM_CLASSES`
   (`src/babylon/engine/simulation_engine.py:328-363`), with the neighbours
   re-verified directly (`reserve_army.py:37` 5.0, Lifecycle 7.0). The registry
   is MEMBERSHIP-only — order is derived by sorting on `position`
   (`:376-377`), so the ClassVar, not tuple index, is the authority. Confirmed.
7. **CONFIRMATION — the libm site and the `log` intrinsic.**
   `src/babylon/models/entities/consciousness.py:290-291` — `entropy -= p *
   math.log(p)` then `entropy / math.log(3)`, reached from `community.py:107`
   via `ideological_contestation`. `log` is declarable
   (`rust/crates/babylon-bsl/src/declarations.rs:110`,
   `DECLARABLE_INTRINSICS = ["exp", "log", "floor"]`, pinned at `:1177`), and the
   tolerance-policy D-record is the right disposition. One further note the
   report does not make: unlike the wage-pressure `exp` sites, this `log` use is
   an **entropy measure over an already-computed distribution**, not a
   stipulated functional form — ADR188 Row 7's prohibition
   (`ai/decisions/ADR188_intrinsic_rider_slate_dispositions.yaml:54-57`) names
   three sigmoid sites and does not reach it. Record that explicitly so a future
   reader does not mistake a libm flag for a doctrine flag.
8. **CONFIRMATION — the RESERVED-LINE flag is correctly placed, and this is one
   of only two inventories in the batch to file one.**
   `src/babylon/models/entities/consciousness.py:356-455`
   (`SUBSTRATE_FLOOR_DEFAULTS`) is National Question consciousness-floor
   calibration by oppressed nationality; "transcribe the numbers verbatim, no
   theory judgment made by the port itself" is the right disposition under
   ADR171. The TickDynamicsSystem and ReserveArmySystem inventories in this
   batch file none at all, and both needed to.

**FINAL VERDICT: BLOCKED — CONFIRMED, and harder than the report states.** Every
computation on the primary data path is blocked, but the headline blocker for
the ternary-consciousness path is **not** slice 3: it is a node-local
list-of-structs storage shape that **no numbered slice serves** (correction 2).
The dyadic-edge blocker (slice 2 for the read, `update-edge`'s one-`f64`
substrate refusal for the write) and the slice-4 attributed-membership
dependency both stand as written, verified live. The
`_get_class_position_name`/solidarity-matrix row drops from PORTABLE WITH
D-RECORD to **D102-blocked absent an int-ordinal `role` encoding**
(correction 1), leaving only the 14-entry substrate-floor constant table, the
5×5 solidarity matrix and the Shannon-entropy formula as genuinely
lane-independent — and the report is correct that none of them can fire without
the membership data the blocked lanes exist to reach. Total dormancy on all 12
canonical scenarios is confirmed three ways with no stale source in the chain,
making this the best-evidenced dormancy finding in the batch.

**COVERAGE NOTE (not inadequacy).** The read is complete (675-line file
line-by-line, dead-vs-reachable call graph separated, both namespace hazards
excluded with evidence). A re-read owes only: the corrected row-1 label
(correction 2), the D102 row (correction 1), and the one-line note that this
system's `log` is a measure and not an ADR188 Row-7 stipulated form
(confirmation 7).
