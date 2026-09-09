# OODASystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `OODASystem` (`src/babylon/engine/systems/ooda.py`, position 14.0,
Action phase) orchestrates a three-layer turn resolution over `ai/ooda/` — a ~6,100-line,
26-module package — but a read-only, whole-repo grep audit finds most of that estate is
**unwired from the tick path**: roughly half the modules (attention thread-manager/observation,
all six `state_ai/*_effects.py` sub-verb resolvers, `constraints.py`, `action_costs.py`,
`lifecycle_capacity.py`, `update_momentum`) have **zero production callers**, and the live NPC
dispatch path collapses almost every non-state organization's selected action to a **blind,
materially-inert `ActionResult`** (only REPRESS/SURVEIL and LEGISLATE do real work). Worse,
every live `RuleBasedStateAI` selection *other than* LEGISLATE gets stamped `ActionType.REPRESS`
regardless of its real sub-verb (`npc_stub.py:495-503`, "Best-match legacy type") — so BRIBE,
INVEST, AUDIT, RAID, etc. all materially resolve as REPRESS. `tools/regression_scenarios.py`
itself declares OODASystem **dormant on every canonical scenario** (`org_count=action_count=
layer0_count=0` on every tick, including the non-canonical `detroit_tri_county` baseline);
only `wayne_county`/`org_probe` seed real Organization nodes, and `org_probe` explicitly
disclaims exercising business logic. The live decision seam (`RuleBasedStateAI.select_action`)
draws Python `random.Random`, a kernel intrinsic BSL has spec'd (§2.8/§3.10) but never
implemented (zero grep hits in `rust/crates/babylon-bsl`/`babylon-tick`) — a real, named
determinism blocker, not a query-lane gap. No `exp`/`log`/`pow`/`sigmoid` appears anywhere in
the live path.

**Verdict:** **BLOCKED as one pack — DEFER to a five-train split**, gated primarily on (1) the
RNG kernel intrinsic (unbuilt), (2) edge-attribute read/write beyond the single `f64` strength
`GraphSubstrate` stores (`update-edge`/`update-hyperedge` refuse by design,
`structural_verbs.rs:16-26,693-700`), and (3) Slice-2 `EdgeRef` field reads for the REPRESSION-
edge/WAGES-value_flow computations; a **thin PORTABLE-NOW slice exists today** (Layer 0
auto-metabolism, initiative scoring minus `momentum`, the static NPC eligibility+priority-queue
skeleton, AGITATE/ASSIMILATE's pure-arithmetic branches) but every canonical scenario is
provably dormant on it, so no conformance oracle exists without hand-built `.bscn` fixtures.

---

## 1. FILE MAP

### The target and its direct orchestration graph (`step()`-reachable, read in full or near-full)

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/ooda.py` | 479 | **The target.** `OODASystem.step()` — Layer 0 → initiative → action-phase dispatch → Layer 3 → turn-resolution publish → event emission. Read completely. |
| `src/babylon/ooda/types.py` | 284 | Data model: `OODAProfile`, `Action`, `ActionResult`, `InitiativeScore`, `ActionCostModifier`, `TurnResolution` — all frozen Pydantic, per-tick-only (never persisted). Read completely. |
| `src/babylon/ooda/cycle_time.py` | 61 | `compute_cycle_time` — four-phase additive OODA-cycle model. Read completely. |
| `src/babylon/ooda/initiative.py` | 201 | `compute_initiative_score`, `resolve_action_order`, `compute_community_embeddedness`, `update_momentum` (**dead — zero callers**), `_institutional_bonus`. Read completely. |
| `src/babylon/ooda/layer0.py` | 72 | `process_layer0` — Business auto-EMPLOY metabolism stub. Read completely. |
| `src/babylon/ooda/layer3.py` | 222 | `process_layer3` — heat/edge-transition/infrastructure propagation + spec-108 corridor-mesh splash. Read completely. |
| `src/babylon/ooda/npc_stub.py` | 508 | `select_npc_actions`, `_try_state_ai_dispatch` (state-AI gate + REPRESS-collapse), `_gather_repress_target_candidates`, `_compute_sparrow_topology_scores`. Read completely. |
| `src/babylon/ooda/action_effects.py` | 582 | `compute_consciousness_delta` (five-factor formula), `resolve_action` (the verb switch: AGITATE/REPRESS+SURVEIL/ASSIMILATE/fascist verbs/fallback), `_bump_repression_edge`, `_propagate_repression_to_class_base`, `_resolve_fascist_verb`. Read completely. |
| `src/babylon/ooda/state_ai/decision.py` | 668 | `RuleBasedStateAI` — the live decision seam: faction-objective scoring, escalation scoring, RNG tiebreak, Sparrow-informed target selection, budget-constrained selection. Read completely. |
| `src/babylon/ooda/state_ai/escalation.py` | 83 | `get_escalation_rank`, `compute_heat_escalation_score` — Gaussian-affinity heat/verb matching. Read completely. |
| `src/babylon/ooda/state_ai/protocols.py` | 49 | `NPCDecisionStrategy` Protocol — its declared `select_action` signature does **not** match `RuleBasedStateAI.select_action`'s real one (structural-typing mismatch, never enforced at runtime — `runtime_checkable` checks method presence, not signature). Read completely. |
| `src/babylon/ooda/_helpers.py` | 89 | `_compute_membership_overlap` — shared by `action_costs.py` (dead) and `action_effects.py` (live). Read completely. |
| `src/babylon/ooda/action_eligibility.py` | 178 | `ELIGIBILITY_MAP` (4 OrgType × 26 ActionType), `check_eligibility` — **NPC-path-only**; never consulted for player-submitted actions. Read completely. |
| `src/babylon/ooda/attention/sparrow.py` | 180 | `analyze_network` — rustworkx degree/betweenness centrality + articulation-point (cutset) analysis over a SOLIDARITY subgraph. Live (imported directly by `npc_stub.py`, not via the package `__init__`). Read completely. |
| `src/babylon/ooda/state_ai/__init__.py` | 102 | Re-export hub for all `state_ai/*_effects.py` symbols. **Itself has zero importers anywhere in `src/babylon`** — confirmed by whole-repo grep. |
| `src/babylon/ooda/__init__.py` | 69 | Package-level re-export hub (`action_costs`, `constraints`, `lifecycle_capacity`, `update_momentum`, …). **Zero `from babylon.ooda import ...` call sites in `src/babylon`** — confirmed by grep. |

### Dead-per-grep sub-estate (defined, tested, never called by production — see §5 for the grep evidence)

| File | Lines | Verb/function surface | Status |
|---|---|---|---|
| `src/babylon/ooda/constraints.py` | 123 | `enforce_action_points`, `enforce_coordination_range`, `apply_autonomy_modifier` | **DEAD** — zero callers outside `tests/`. The AP-budget/coordination-range/autonomy-tradeoff enforcement `specs/032` describes is entirely unexercised; `OODAProfile.action_points`/`.coordination_range`/`.autonomy` are read nowhere in the live path. |
| `src/babylon/ooda/action_costs.py` | 172 | `compute_action_cost` (community-modified AP cost, contradiction-axis surcharge) | **DEAD** — zero callers outside `tests/`. |
| `src/babylon/ooda/lifecycle_capacity.py` | 76 | `compute_lifecycle_modifier`, `elder_legitimacy_bonus` | **DEAD** — zero callers outside `tests/`. |
| `src/babylon/ooda/attention/thread_manager.py` | 228 | `advance_thread_phase`, `allocate_threads`, `update_thread_tick` | **DEAD** — only imported by `attention/__init__.py`, itself unimported elsewhere. |
| `src/babylon/ooda/attention/observation.py` | 123 | `build_g_observed`, `compute_observation_ceiling` | **DEAD** — same as above. |
| `src/babylon/ooda/state_ai/administer_effects.py` | 155 | `resolve_audit`, `resolve_fund`, `resolve_staff` | **DEAD** — zero callers outside `tests/`/`state_ai/__init__.py`. |
| `src/babylon/ooda/state_ai/co_opt_effects.py` | 212 | `resolve_bribe`, `resolve_divide`, `resolve_propagandize`, `compute_incorporate_probability` | **DEAD** — same. |
| `src/babylon/ooda/state_ai/repress_effects.py` | 380 | `resolve_infiltrate`, `resolve_liquidate`, `resolve_prosecute`, `resolve_raid`, `compute_raid_consciousness_effect` | **DEAD** — same. |
| `src/babylon/ooda/state_ai/territory_effects.py` | 491 | `resolve_displace`, `resolve_invest`, `resolve_neglect`, `resolve_scorched_earth`, `resolve_strategic_withdrawal`, `resolve_eviction_cascade`, `compute_heat_accumulation/_decay`, `compute_propagandize_effect`, `compute_scorched_earth_legitimacy`, `assess_territory_threat`, `check_recruit_effectiveness` | **DEAD** — same. |
| `src/babylon/ooda/state_ai/legislate_effects.py` | 75 | `consume_legal_framework_effects` | **DEAD** — same. |
| `src/babylon/ooda/state_ai/observability.py` | 178 | `create_observable_action`, `create_territory_observables`, `resolve_counter_intel` | **DEAD** — same. |
| `src/babylon/ooda/state_ai/faction_dynamics.py` | 525 | `apply_fascist_overrides`, `apply_material_condition_shift`, `apply_player_action_shift`, `apply_repression_failure_shift`, `compute_stability` — **DEAD** from OODA's own call graph. `renormalize_faction_balance` — **LIVE, but called by `engine/systems/electoral.py:1029`, a different System**, not by `OODASystem`. |

**Total dead-per-grep module weight: 2,568 lines** (constraints+action_costs+lifecycle_capacity
371 + attention thread_manager/observation 351 + the six `state_ai/*_effects.py` modules minus
`renormalize_faction_balance`'s home 1,516 + `state_ai/__init__.py`'s 102-line hub, which only
re-exports the above). This is roughly **42% of the ~6,100-line `ooda/` package** by line count.

### Adjacent estates OODASystem orchestrates but this inventory does not deep-dive (out of scope, per mission scope)

| Estate | Lines | Role | Dormancy |
|---|---|---|---|
| `src/babylon/engine/actions/` (player-verb registry: `__init__.py` + `aid.py`, `attack.py`, `campaign.py`, `educate.py`, `investigate.py`, `mobilize.py`, `move.py`, `negotiate.py`, `reproduce.py`, `_mass_work.py`, `_capability.py`, `build.py`) | 1,814 | `resolve_player_action` dispatches through `VERB_RESOLVERS` (9 of 26 `ActionType`s: EDUCATE, RECRUIT→`resolve_reproduce`, ATTACK_INFRASTRUCTURE, PROTEST→`resolve_mobilize`, PROPAGANDIZE→`resolve_campaign`, PROVIDE_SERVICE→`resolve_aid`, MAP_NETWORK→`resolve_investigate`, MOVE, PROPOSE_ALLIANCE→`resolve_negotiate`) — invoked from `_resolve_for_organization` (`ooda.py:311-332`) only when `context.persistent_data["player_actions"]` names the acting org. `educate.py`/`campaign.py` internally call `action_effects.resolve_action`. | **Dormant on every canonical/headless scenario** — no `qa:regression` scenario, `single_county`, `org_probe`, or `detroit_tri_county` ever populates `player_actions`; this path only fires under a live played session (Bevy client). Deserves its own, separate port train — out of this inventory's computation catalog. |
| `src/babylon/config/defines/ooda.py` | 490 | `OODADefines` — cycle-time, initiative, action-cost, consciousness, autonomy, Layer-3 coefficients + `validate_derivations` cross-check. Read completely. |
| `src/babylon/config/defines/state_apparatus.py` (`StateApparatusAIDefines` portion, lines 12-490) | ~478 | State-AI coefficients: faction dynamics, fascist convergence, attention threads, budget, escalation ladder, territory/spatial/co-opt/administer/repress/legislate effect magnitudes — most of these coefficients feed the **dead** `state_ai/*_effects.py` modules. Read completely (`InstitutionDefines`, lines 492-556, belongs to a different System and is out of scope). |
| `src/babylon/config/defines/reactionary.py` | 160 | `ReactionaryDefines` — spec-071 fascist-verb coefficients (`pogrom_repression_increment`, `pogrom_wealth_destruction`, `vigilantism_repression_increment`, `lockout_wage_attenuation`), consumed by `action_effects._resolve_fascist_verb`. Read completely. |

### Supporting model/enum/kernel files (spot-read for field types and call confirmation)

| File | Relevant content |
|---|---|
| `src/babylon/models/enums/actions.py:32-92` | `ActionType` — 26 values (21 base + MOVE + POGROM/LOCKOUT/VIGILANTISM/RED_BROWN_COUP). |
| `src/babylon/models/enums/organizations.py:63-123` | `StateActionType` — 6 top-level verbs (ADMINISTER/DEVELOP/RESEARCH/CO_OPT/REPRESS/WITHDRAW) + 21 sub-verbs. |
| `src/babylon/models/enums/social.py:88-103` | `OrgType` — 4 values (STATE_APPARATUS/BUSINESS/POLITICAL_FACTION/CIVIL_SOCIETY). |
| `src/babylon/models/entities/state_apparatus_ai.py` | `StateAction`, `FactionBalance`, `StateBudget`, `VERB_CHILDREN` (parent→sub-verb hierarchy). |
| `src/babylon/models/entities/organization.py:150-340` | `Organization` base + `StateApparatus`/`Business` subtypes. `rng_seed`'s own docstring (line ~326) names the OS-entropy fallback "**a real non-determinism bug**" verbatim. `momentum` and `counter_intel_score` are **not declared fields anywhere on this model** (grep-confirmed zero hits) — see §3. |
| `src/babylon/domain/organizations/consciousness.py` | `tendency_modifier` — live, used by `compute_consciousness_delta`. |
| `src/babylon/domain/organizations/composition.py` | `lifecycle_composition`, `effective_capacity` — used only by the **dead** `lifecycle_capacity.py`. |
| `src/babylon/domain/geography/corridor_mesh.py` | `apply_uniform_territory_splash` — live via `layer3.py`, conditional on `corridor_mesh`/`transport_defines` both present (spec-108; `TransportSystem` @9.5 is default-OFF per `TransportDefines.enabled=False`, so this branch is itself dormant on every canonical scenario — `tools/regression_scenarios.py:2729-2735`). |
| `src/babylon/engine/systems/policy.py`, `src/babylon/domain/politics/policy.py` | `enqueue_agenda_item`, `PolicyAgendaItem` — live via `_enqueue_legislate` (`ooda.py:385-438`). |
| `src/babylon/engine/simulation_engine.py:328-363` | `_SYSTEM_CLASSES` — confirms tick position 14.0 (16th of 34 systems, after `MetabolismSystem` @13.0, before `FactionInfluenceSystem` @14.5). |
| `src/babylon/sentinels/vocabulary/registry.py:469-499` | `SentinelExemption` rows for `ooda_profile` and `counter_intel_score` — the vocabulary sentinel's own documentation that neither production seeder ever stamps them. |
| `ai/decisions/ADR184_capacity_belongs_to_organizations.yaml` | The **forward** (Rust-target) capacity model — unifies repression + revolutionary-action budgets under one `Capacity`/`Candidate`/`allocate` construct owned by an `Organization`. Explicitly notes "the whole Lane A estate still has ZERO external callers" — this is Rust-side (`rust/crates/babylon-graph/src/capacity.rs`), **not yet wired to anything**, and structurally different from the frozen Python's two parallel systems (`OODAProfile.action_points` AP budget + the separate `StateBudget` used only by `RuleBasedStateAI`). See §6. |
| `tools/regression_scenarios.py:2612-2704, 2750-2760` | `org_probe` scenario definition + the `COVERAGE_GAPS_DATA` row declaring OODASystem dormant on every canonical scenario. See §5. |
| `src/babylon/engine/scenarios/org_probe.py`, `_legacy_wayne.py:605-662` | The only two scenario factories that seed real `NodeType.ORGANIZATION` rows. |

---

## 2. COMPUTATION CATALOG (execution order, `ooda.py:108-265`)

### 2.1 — Layer 0: automatic Business metabolism (`process_layer0`, `layer0.py:21-69`)

- **(a)** Every `BUSINESS`-type organization auto-generates one EMPLOY `ActionResult` per tick, representing "ongoing economic activity," before initiative ordering runs.
- **(b)** No formula — a pure stamp: `Action(org_id, ActionType.EMPLOY, target_id=territory_ids[0] or org_id)`, `ActionResult(success=True, direct_effects={"auto_metabolism": True}, events_generated=[ORGANIZATIONAL_ACTION])` (`layer0.py:55-66`). **No graph mutation whatsoever** — `direct_effects` is discarded after the tick (per `types.py`'s own docstring: "computed per-tick and never stored permanently").
- **(c) Reads:** `NodeType == "organization"` + `org_type == OrgType.BUSINESS.value` (`layer0.py:42-45`), `territory_ids` (static Organization field).
- **(d) Writes:** none.
- **(e) Defines:** none.
- **(f) Events:** `EventType.ORGANIZATIONAL_ACTION` (per Business org, folded into the tick's summary event later — see 2.9).

### 2.2 — Org-node collection and initiative scoring (`ooda.py:132-170`, `cycle_time.py`, `initiative.py:22-67`)

- **(a)** Every organization node gets a per-tick "how fast/first can I act" score from four independent inputs: OODA-cycle speed, an institutional (jurisdiction) bonus, counter-intel capability, community embeddedness, and momentum; the tick's action order is these scores sorted descending.
- **(b)** Cycle time (`cycle_time.py:17-39`): `observe_time = base_observe_time + sensor_latency * latency_weight`; `orient_time = max(base_orient_time * (1.0 - ideological_coherence * coherence_weight), orient_time_floor)`; `decide_time = decision_base(mode) * (1.0 + bureaucratic_depth * depth_weight)`; `act_time = base_act_time`; `cycle_time = observe_time + orient_time + decide_time + act_time`. `decision_base` is a 4-way dict lookup (AUTOCRATIC/DELEGATE/DEMOCRATIC/CONSENSUS) at `cycle_time.py:42-58`.
  Initiative score (`initiative.py:22-67`): `speed = initiative_weight_speed * (1.0 / cycle_time)`; `institutional = initiative_weight_institutional * inst_bonus` where `inst_bonus` is a 4-way jurisdiction lookup (`_institutional_bonus`, `initiative.py:174-193`) or `institutional_bonus_nonstate` (0.0) for non-state orgs; `counterintel = initiative_weight_counterintel * counter_intel_score`; `embeddedness = initiative_weight_embeddedness * community_embeddedness`; `momentum_val = initiative_weight_momentum * momentum`; `score = speed + institutional + counterintel + embeddedness + momentum_val`.
  Embeddedness (`compute_community_embeddedness`, `initiative.py:82-150`): walks `org.territory_ids` → TENANCY-linked `social_class` members → `share = |members with a non-empty community_memberships| / |members|`, sorted-iteration deterministic, clamped `[0,1]`. Its own docstring states the value is "structurally 0.0 in every real game today" — `CommunitySystem.step` never populates `community_memberships` (`sentinels/seam/registry.py:1969-1991`, `liveness_class=STRUCTURALLY_IMPOSSIBLE`).
  Sort (`resolve_action_order`, `initiative.py:70-79`): `sorted(scores, key=lambda s: (-s.score, s.org_id))` — descending score, ascending id tiebreak.
- **(c) Reads:** `ooda_profile` (nested dict → `OODAProfile`, **never seeded by any production scenario** — see §5), `org_type`, `jurisdiction` (state orgs only), `counter_intel_score` (**undeclared field, never seeded** — see §3), `momentum` (**undeclared field, never written anywhere** — see §3), TENANCY edges + `community_memberships` (dead, see above).
- **(d) Writes:** none (score is per-tick, discarded).
- **(e) Defines:** `ooda.base_observe_time` (1.0), `latency_weight` (0.5), `base_orient_time` (2.0), `coherence_weight` (0.6), `orient_time_floor` (0.1), `base_act_time` (1.0), `depth_weight` (0.4), `decision_mode_base_{autocratic,delegate,democratic,consensus}` (1.0/2.0/3.0/5.0), `initiative_weight_{speed,institutional,counterintel,embeddedness,momentum}` (2.0/1.0/1.5/1.0/0.5), `institutional_bonus_{federal,state,local,nonstate}` (5.0/3.0/1.5/0.0) — `defines.yaml:568-646` (`ooda:` block); all `[C]`-tagged theory-derived constants per `ooda.py` (the config module)'s own docstrings, not `[S]`ynthetic.
- **(f) Events:** none.

### 2.3 — NPC action selection: state-AI gate (`select_npc_actions` / `_try_state_ai_dispatch`, `npc_stub.py:71-505`)

- **(a)** A `STATE_APPARATUS` org with a populated `faction_balance` delegates to `RuleBasedStateAI`; every other org (including a `STATE_APPARATUS` **without** `faction_balance`) falls through to a static priority queue (2.4).
- **(b)** Gate: `faction_data = org_attrs.get("faction_balance")`; `if faction_data is None: return None` (fall through) (`npc_stub.py:374-377`).
- **(c) Reads:** `org_type`, `faction_balance` (nested → `FactionBalance`), `state_budget` (nested → `StateBudget`, or a hardcoded default: `revenue=100.0, available=100.0, allocated={ADMINISTER:15, DEVELOP:15, RESEARCH:10, CO_OPT:20, REPRESS:30, WITHDRAW:10}, imperial_rent_pool=50.0` at `npc_stub.py:399-411` — a **bare literal default, not a `services.defines` value**), `heat` (default 0.3, `npc_stub.py:413`), `rng_seed` (`Optional[int]`, `npc_stub.py:414`).
- **(d) Writes:** none directly (delegates to 2.5-2.6).
- **(e) Defines:** `services.defines.state_ai` (`StateApparatusAIDefines`) threaded through when available; else a fresh `GameDefines().state_ai` with a `_log.debug` note that this branch "should be unreachable in production" (`npc_stub.py:416-435`).
- **(f) Events:** none directly.

### 2.4 — NPC action selection: static priority queue (`select_npc_actions`, `npc_stub.py:106-143`)

- **(a)** Every non-state-AI-dispatched org greedily takes its org-type's fixed priority list of verbs until its `OODAProfile.action_points` budget (default 3) runs out, gated by `check_eligibility`.
- **(b)** `_NPC_PRIORITIES` (`npc_stub.py:41-68`): STATE_APPARATUS→[SURVEIL, REPRESS, INFILTRATE, MAP_NETWORK, COUNTER_INTEL]; POLITICAL_FACTION→[EDUCATE, ORGANIZE, AGITATE, RECRUIT, FUNDRAISE]; CIVIL_SOCIETY→[PROVIDE_SERVICE, EDUCATE, ORGANIZE, FUNDRAISE, BUILD_INFRASTRUCTURE]; BUSINESS→[EMPLOY, FUNDRAISE, DENOUNCE]. Loop: `if not check_eligibility(...): continue`; `cost = defines.get_base_cost(action_type.value)`; `if cost > remaining_ap: continue`; else append and `remaining_ap -= cost`.
- **(c) Reads:** `org_type`, `ooda_profile.action_points` (default 3, dict-path, not the `OODAProfile` model's validated default), `violence_capacity`/`surveillance_capacity`/`consciousness_tendency`/`is_institution` (only for the REPRESS/SURVEIL/ASSIMILATE eligibility special cases, `action_eligibility.py:153-170`).
- **(d) Writes:** none (produces `Action` objects only).
- **(e) Defines:** `ooda.base_cost_*` (25 int fields, `defines.yaml:568-646`) via `OODADefines.get_base_cost` (`ooda.py`(defines):420-463).
- **(f) Events:** none directly. **Downstream fate (2.9):** every action from this path whose `action_type` is NOT in `{REPRESS, SURVEIL}` resolves as a **materially-inert stamp** — `ActionResult(success=True, events_generated=[ORGANIZATIONAL_ACTION])` (`ooda.py:374-381`), no consciousness delta, no graph write. This means, verbatim, in the frozen system: **no `POLITICAL_FACTION`, `CIVIL_SOCIETY`, or `BUSINESS` NPC action ever has a material effect** — EDUCATE/ORGANIZE/AGITATE/RECRUIT/FUNDRAISE/PROVIDE_SERVICE/BUILD_INFRASTRUCTURE/EMPLOY/DENOUNCE are all cosmetic when NPC-selected. The *code* that would compute their real effects exists (`action_effects.resolve_action`'s AGITATE branch, its EDUCATE/ORGANIZE/RECRUIT/PROVIDE_SERVICE/FUNDRAISE fallback via `compute_consciousness_delta`) but is unreachable from this path — only from the player-verb registry (out of scope, §1), and even there `PROPAGANDIZE`/`AGITATE`/`ORGANIZE`/`FUNDRAISE`/`RECRUIT`/`DENOUNCE`/`EMPLOY`/`STRIKE`/`EXPROPRIATE` have **no player resolver either** (`VERB_RESOLVERS` has only 9 keys, `engine/actions/__init__.py:59-67`) — a player action of those types dispatches to a loud `success=False` failure (`engine/actions/__init__.py:91-97`). Port-as-is: this dead-verb surface must be transcribed faithfully, not silently "fixed" into working verbs.

### 2.5 — `RuleBasedStateAI.select_action` — the live decision seam (`state_ai/decision.py:461-604`)

- **(a)** OBSERVE+ORIENT: generate one candidate `StateAction` per affordable sub-verb. DECIDE: score each by a faction-weighted objective plus an escalation-ladder affinity plus a random tiebreak; sort descending. Then pick ONE target — the highest `heat × visibility [× Sparrow topology]` visible non-state org — using the top-scored candidate's sub-verb to choose the Sparrow targeting mode. ACT: greedily fund top-scored actions within budget, all aimed at the one resolved target.
- **(b)** Candidate generation (`_generate_candidates`, `decision.py:359-412`): one `StateAction` per `(parent, child)` in `VERB_CHILDREN` whose `_VERB_COSTS[child]` (a hardcoded dict, `decision.py:37-67`, NOT `services.defines`-sourced) `<= budget_available`; `legitimacy_cost` from `_LEGITIMACY_COSTS` (`decision.py:74-98`, also hardcoded), floored to `±minimum_effect_floor` if `0 < |cost| < floor` (`decision.py:397-399`).
  Faction-objective scoring (`decision.py:106-245`): three hand-tuned piecewise functions (`finance_capital_objective`, `security_state_objective`, `settler_populist_objective`) each mapping `(verb, heat)` to a bare-literal score via `if/elif` chains (e.g. FC: `CO_OPT→1.5, DEVELOP→1.2, ADMINISTER→0.8, RESEARCH→0.6, REPRESS→-0.5+heat*1.0, WITHDRAW→-1.0`, plus `legitimacy_penalty = legitimacy_cost * 2.0`); `score_action` is the balance-weighted sum: `fc_weight*fc + ss_weight*ss + sp_weight*sp`.
  Escalation scoring (`escalation.py:18-76`): `esc_rank = get_escalation_rank(sub_verb, defines)` (linear scan of `defines.escalation_ladder`, a 16-entry ordered string list); `esc_score = compute_heat_escalation_score(heat, esc_rank, max_rank) = max(0, 1 - |heat - esc_rank/max_rank|) * 2.0` if on the ladder, else a neutral `0.5`.
  Combined score + RNG tiebreak (`decision.py:553`): `combined = faction_score + esc_score + rng.uniform(0.0, 0.01)`, where `rng = random.Random(rng_seed)` — **`rng_seed` comes from `org_attrs.get("rng_seed")`, which is `None` unless the Organization model explicitly sets it**; the model's own docstring (`models/entities/organization.py`, `StateApparatus.rng_seed`) calls the `None`-fallback "**a real non-determinism bug**" verbatim.
  Target selection (`select_repress_target`, `decision.py:258-351`): filter candidates to `heat > 0` and `id != org_id`; if any eligible candidate has a positive Sparrow topology score for the winning sub-verb, sort by `(-(heat*visibility*topology_score), id)`, else by `(-(heat*visibility), id)`; `None` if no eligible candidate.
- **(c) Reads:** `faction_balance.{finance_capital,security_state,settler_populist}`, `state_budget.available`, `heat` (org's own, default 0.3), `rng_seed`, `target_candidates` (from 2.6), `sparrow_topology_scores` (from 2.7), `defines.escalation_ladder`/`.minimum_effect_floor`/`.actions_per_tick`/`.god_mode_enabled`.
- **(d) Writes:** none (returns `list[StateAction]`, consumed by 2.8).
- **(e) Defines:** `state_ai.minimum_effect_floor` (0.02), `state_ai.actions_per_tick` (1), `state_ai.escalation_ladder` (16-entry list) — `defines.yaml:755-846`. `_VERB_COSTS`/`_LEGITIMACY_COSTS` (`decision.py:37-98`) are **module-level Python dict literals, NOT `defines.yaml`-sourced** — a defines-bypass distinct from every other coefficient in this system.
- **(f) Events:** none directly.

### 2.6 — REPRESS-target candidate gathering (`_gather_repress_target_candidates`, `npc_stub.py:146-197`)

- **(a)** Enumerate every non-state-apparatus `organization` node (excluding the acting org itself) as a `(id, heat)` candidate pair.
- **(b)** Plain filter-and-collect over `graph.nodes(data=True)`, no arithmetic.
- **(c) Reads:** `_node_type`, `org_type`, `heat` (per candidate). Deliberately **excludes `SocialClass` nodes** — its own docstring cites that `SocialClass` has no `heat` field and is `extra="forbid"` frozen, so writing one would break `WorldState.from_graph()` round-trip (the same landmine class documented for Territory's `infrastructure`).
- **(d) Writes:** none.
- **(e) Defines:** none.
- **(f) Events:** none.

### 2.7 — Sparrow topological targeting scores (`_compute_sparrow_topology_scores`, `npc_stub.py:200-347`; `attention/sparrow.py:analyze_network`)

- **(a)** Constitution I.21 (Sparrow Three-Targeting-Modes): score every REPRESS-target candidate by its structural role in the observed SOLIDARITY subgraph — degree+betweenness centrality (RAID/LIQUIDATE), articulation-point membership (INFILTRATE→cutset), inverse-degree isolation (SURVEIL).
- **(b)** Builds a `BabylonGraph` subgraph over just the candidate ids + their SOLIDARITY edges, then calls `analyze_network` (rustworkx centrality + `rustworkx`-derived articulation points, deterministic, id-sorted internally per the docstring). `centrality = degree + betweenness`; `cutset = 1.0 if node in known_cutsets else 0.0`; `isolation = 1.0 - degree`.
- **(c) Reads:** candidate ids, SOLIDARITY edges among them (`weight`).
- **(d) Writes:** none (builds a throwaway local subgraph, never mutates the real graph).
- **(e) Defines:** none directly (rustworkx algorithm parameters are library-internal).
- **(f) Events:** none. **Dormancy note:** its own docstring states the SOLIDARITY subgraph "is empty today — no verb yet writes an org-to-org SOLIDARITY edge" (`engine/actions/_mass_work.py` only writes org→social_class SOLIDARITY), so `centrality`/`cutset` are structurally all-zero in every live run; only `isolation` (uniformly 1.0 on empty input) has any effect, and that effect is a no-op multiplier (see the function's own docstring, `npc_stub.py:283-291`).

### 2.8 — NPC action → legacy `Action` conversion (`_try_state_ai_dispatch`, `npc_stub.py:459-505`)

- **(a)** Every selected `StateAction` becomes a legacy `Action`. LEGISLATE keeps its identity and routes to the policy agenda (2.10). **Every other sub-verb — BRIBE, INCORPORATE, DIVIDE, FUND, STAFF, AUDIT, REVOKE, INVEST, REZONE, DISPLACE, NEGLECT, PURSUE_TECH, DEPLOY_TECH, SURVEIL, INFILTRATE, RAID, PROSECUTE, LIQUIDATE, STRATEGIC_WITHDRAWAL, TACTICAL_RETREAT, SCORCHED_EARTH — is stamped `ActionType.REPRESS`**, discarding its real sub-verb identity for material-effect purposes (the comment reads "Best-match legacy type," `npc_stub.py:498`).
- **(b)** No arithmetic — a pure re-stamp: `Action(org_id, ActionType.REPRESS, target_id=sa.target_id or org_id, action_point_cost=1, budget_cost=sa.budget_cost)` (`npc_stub.py:495-503`). **Verbatim defect, transcribe faithfully (port-as-is law):** because `ActionType.REPRESS` is exactly the type gated into `_MATERIALLY_RESOLVED_NPC_VERBS`, EVERY non-LEGISLATE state-AI selection — including BRIBE, INVEST, AUDIT, FUND — materially resolves as `_resolve_repressive` (2.9): a REPRESS-shaped `repression_faced` bump + REPRESSION edge stamp + REVOLUTIONARY-tendency CI backfire, regardless of what the AI actually "decided."
- **(c) Reads:** `sa.sub_verb`, `sa.target_id`, `sa.budget_cost`, `balance.dominant_faction` (LEGISLATE-only, for the axis-by-faction proxy).
- **(d) Writes:** none directly (produces the `Action` list consumed by `ooda.py`'s dispatch loop).
- **(e) Defines:** none.
- **(f) Events:** none directly.

### 2.9 — Materially-resolved-verb dispatch and `action_effects.resolve_action` (`ooda.py:346-381`; `action_effects.py:127-186, 376-515`)

- **(a)** Only `REPRESS`/`SURVEIL` (`_MATERIALLY_RESOLVED_NPC_VERBS`, `ooda.py:67-69`) actually call the effects switch; everything else in the NPC path (2.4/2.8's collapsed verbs) is the blind no-op stamp already described.
- **(b)** `resolve_action`'s switch (`action_effects.py:152-186`): `AGITATE → _resolve_agitate` (returns `direct_effects={"contestation_delta": agitation_contestation_delta}`, no graph write); `REPRESS|SURVEIL → _resolve_repressive`; `ASSIMILATE → _resolve_assimilate`; POGROM/LOCKOUT/VIGILANTISM → `_resolve_fascist_verb`; else → the generic `compute_consciousness_delta` fallback (reachable only via player dispatch, §1).
  `_resolve_repressive` (`action_effects.py:392-488`): `backfire_delta = min(action_base(action_type) * base_credibility, max_ci_delta_per_tick)`, `ConsciousnessDelta(REVOLUTIONARY, backfire_delta)`; if target is a `social_class` node, `repression_faced = min(1.0, current + increment)` where `increment = repress_heat_delta (0.15)` or `surveil_heat_delta (0.05)`; if target is an `organization` node, the SAME increment is split evenly across its SOLIDARITY-linked class base (`_propagate_repression_to_class_base`, `action_effects.py:226-308`: `split = increment / len(connected_classes)`, applied id-sorted); either way `_bump_repression_edge` (`action_effects.py:189-223`) stamps/strengthens a REPRESSION edge, `weight = min(1.0, existing.weight + increment)` or `min(1.0, increment)` if new — **skips (does not clobber) if a DIFFERENT edge type already occupies that node pair**, since the graph is not a multigraph.
  `_resolve_assimilate` (`action_effects.py:491-515`): `ci_raw = -(action_base_assimilate * base_credibility)`, clamped to `max(-max_ci_delta_per_tick, ci_raw)`.
  `_resolve_fascist_verb` (`action_effects.py:311-373`): POGROM/VIGILANTISM — `repression_faced = min(1.0, current + increment)` (increment = `pogrom_repression_increment` 0.2 or `vigilantism_repression_increment` 0.1) + `_bump_repression_edge`; POGROM additionally destroys wealth: `new_wealth = current_wealth * (1.0 - pogrom_wealth_destruction)` (0.1). LOCKOUT — for every WAGES edge targeting the target: `value_flow *= (1.0 - lockout_wage_attenuation)` (0.5).
  `compute_consciousness_delta` (`action_effects.py:39-124`, the five-factor formula, reachable only via player dispatch): short-circuits to a zero delta if `cadre_level == 0` or `cohesion == 0`; else `overlap = _compute_membership_overlap(...)`, `effective_credibility = base_credibility * max(overlap, 0.01)`, `base_delta = tendency_modifier(tendency) * cadre_level * cohesion * effective_credibility`, `scaled = base_delta * action_base`; EDUCATE gets a `*= agitation_educate_bonus (1.5)` multiplier when target `ideological_contestation > contestation_threshold (0.3)`; a doctrine-tag theory bonus (`Doctrine Tree Unit 6b`, ADR073 — **RESERVED-LINE, doctrine content**) multiplies by `1 + theory_bonus_per_class_analysis * min(class_analysis, 10.0)` when the org's `doctrine_tags[CLASS_ANALYSIS] > 0`; final clamp `max(-max_ci_delta_per_tick, min(max_ci_delta_per_tick, scaled))`.
- **(c) Reads:** `repression_faced`/`wealth` (target node), REPRESSION edges (`get_edge`/`has_edge`), WAGES edges (`query_edges`, `value_flow`), SOLIDARITY edges (org→class), `consciousness_tendency`, `cadre_level`, `cohesion`, `legitimacy`/`legal_standing`/`employment_count`/`community_workforce` (credibility derivation, `_derive_credibility_from_attrs`, `action_effects.py:536-576`), `ideological_contestation` (target), `doctrine_tags` (**RESERVED-LINE**).
- **(d) Writes:** `repression_faced` (social_class, clamped `[0,1]`), `wealth` (POGROM target, unclamped), REPRESSION edge `weight` (clamped `[0,1]`), WAGES edge `value_flow` (unclamped multiplicative attenuation).
- **(e) Defines:** `ooda.repress_heat_delta` (0.15), `surveil_heat_delta` (0.05), `max_ci_delta_per_tick` (0.05), `action_base_{repress,surveil,assimilate,educate,agitate,...}`, `agitation_educate_bonus` (1.5), `contestation_threshold` (0.3), `agitation_contestation_delta` (0.1) — `defines.yaml:568-646`; `reactionary.pogrom_repression_increment` (0.2), `.pogrom_wealth_destruction` (0.1), `.vigilantism_repression_increment` (0.1), `.lockout_wage_attenuation` (0.5) — `defines.yaml:927-949`; `doctrine.theory_bonus_per_class_analysis` (**RESERVED-LINE**, doctrine coefficient, not read directly by this inventory).
- **(f) Events:** `EventType.ORGANIZATIONAL_ACTION` (generic branches), `EventType.STATE_REPRESSION`/`STATE_SURVEILLANCE` (`_resolve_repressive`), `EventType.POGROM`/`LOCKOUT`/`VIGILANTISM` (`_FASCIST_VERBS` dict, `action_effects.py:32-36`).

### 2.10 — LEGISLATE enqueue (`_enqueue_legislate`, `ooda.py:385-438`)

- **(a)** Draft a policy-agenda item under the TOP claims-holder sovereign of the acting state org's territory; a loud failure if the org has no claimed territory.
- **(b)** No arithmetic beyond `magnitude = float(services.defines.politics.policy_default_magnitude)` (a straight defines read).
- **(c) Reads:** `territory_ids`, `graph.query_territory_claims(territory_id)` (first row = top claimant).
- **(d) Writes:** enqueues a `PolicyAgendaItem` onto the sovereign's agenda register (consumed by `PolicySystem` @17.47, same tick's Consequences phase).
- **(e) Defines:** `politics.policy_default_magnitude`.
- **(f) Events:** `EventType.ORGANIZATIONAL_ACTION` on success; a loud `ActionResult(success=False, failure_reason=...)` on no-sovereign.

### 2.11 — Layer 3: consequence propagation (`process_layer3`, `layer3.py:25-219`)

- **(a)** Three graph-mutating passes over the tick's full `ActionResult` list: heat bump from REPRESS/SURVEIL, edge-type transition (TRANSACTIONAL→SOLIDARISTIC) from ORGANIZE, infrastructure delta from BUILD/ATTACK_INFRASTRUCTURE (+ optional corridor-mesh splash).
- **(b)** `_propagate_heat` (`layer3.py:75-118`): `new_heat = min(1.0, current_heat + heat_delta)`, `heat_delta = repress_heat_delta (0.15)` or `surveil_heat_delta (0.05)`. `_propagate_edge_transitions` (`layer3.py:121-155`): `if edge_type == TRANSACTIONAL: edge_type = SOLIDARISTIC` — a discrete re-stamp, no numeric formula. `_propagate_infrastructure` (`layer3.py:158-219`): `delta = build_infrastructure_delta (0.1)` or `-attack_infrastructure_delta (0.1)`; `new = max(0.0, min(1.0, current + delta))`; when `corridor_mesh`/`transport_defines` both present, also calls `apply_uniform_territory_splash` with `splash_delta = build_splash_condition_repair` or `-attack_splash_condition_damage` (spec-108, `TransportSystem` default-OFF — dormant, §1).
- **(c) Reads:** `heat`, `infrastructure` (target node, both default-provided if absent), `edge_type` (org→target edge, if it exists).
- **(d) Writes:** `heat` [0,1] clamped, `infrastructure` [0,1] clamped, `edge_type` (discrete re-stamp TRANSACTIONAL→SOLIDARISTIC).
- **(e) Defines:** `ooda.repress_heat_delta` (0.15), `surveil_heat_delta` (0.05), `build_infrastructure_delta` (0.1), `attack_infrastructure_delta` (0.1) — `defines.yaml:638-641`; `transport.build_splash_condition_repair`/`attack_splash_condition_damage` (conditional, not read by canonical scenarios).
- **(f) Events:** none (this function only mutates state and returns a summary-count dict).

### 2.12 — Turn-resolution publish + event emission (`ooda.py:220-265`)

- **(a)** Bundle every layer's results into a `TurnResolution`, publish it onto `context.persistent_data["turn_resolution"]`, then emit first-class events for the five reactionary/state-repression event types plus one aggregate summary event.
- **(b)** No arithmetic — a Pydantic `model_dump(mode="json")` + a `for` loop over `action_phase_results` filtering `events_generated` against `_FIRST_CLASS_ACTION_EVENTS` (`ooda.py:46-54`: POGROM, LOCKOUT, VIGILANTISM, STATE_REPRESSION, STATE_SURVEILLANCE).
- **(c) Reads:** `layer0_results`, `initiative_order`, `action_phase_results`, `layer3_effects`.
- **(d) Writes:** `context.persistent_data["turn_resolution"]` (a JSON-mode dict — NOT part of the hashed graph state).
- **(e) Defines:** none.
- **(f) Events:** the five first-class events above (payload: `org_id`, `target_id`, `**direct_effects`), plus one `EventType.ORGANIZATIONAL_ACTION` summary event every tick (payload: `layer0_count`, `action_count`, `org_count`).

**Total distinct `EventType` values emitted by OODASystem: 6** — `ORGANIZATIONAL_ACTION`, `POGROM`, `LOCKOUT`, `VIGILANTISM`, `STATE_REPRESSION`, `STATE_SURVEILLANCE` (grep-confirmed across `ooda.py`, `layer0.py`, `action_effects.py`; `layer3.py` emits none).

---

## 3. TYPE INVENTORY

| Attribute | Node/edge type | Python model type | Domain | Category |
|---|---|---|---|---|
| `org_type` | ORGANIZATION | `OrgType` (StrEnum, 4 members) — Pydantic discriminated-union discriminator | closed set | **Enum discriminant** |
| `consciousness_tendency` | ORGANIZATION | `ConsciousnessTendency` (StrEnum) | closed set | **Enum discriminant** |
| `legal_standing` | ORGANIZATION (StateApparatus) | `LegalStanding` (StrEnum) | closed set | **Enum discriminant** |
| `jurisdiction` | ORGANIZATION (StateApparatus) | `JurisdictionLevel` (StrEnum) | closed set | **Enum discriminant** |
| `decision_mode` | OODAProfile (nested) | `DecisionMode` (StrEnum, 4 members) | closed set | **Enum discriminant** |
| `cohesion`, `cadre_level`, `heat`, `violence_capacity`, `surveillance_capacity` | ORGANIZATION | `Probability` (`Annotated[float, ge=0, le=1]`) | `[0,1]` | unit-interval |
| `ideological_coherence`, `analytical_capacity`, `autonomy` | OODAProfile (nested) | `float` (Field `ge=0,le=1`) | `[0,1]` | unit-interval |
| `sensor_latency`, `action_points`, `coordination_range` | OODAProfile (nested) | `int` (Field-bounded) | `[0,10]`/`[0,20]`/`[0,100]` | bounded int |
| `budget` | ORGANIZATION | `Currency` (= `Annotated[float, ge=0]`, plain unbounded-above float, same non-BSL-Currency caveat as Territory's `rent_level`) | `[0,∞)` | **unbounded real, money-semantic** |
| `wealth` | ORGANIZATION (POGROM target) | `float` (untyped read in `action_effects.py`, no declared `Currency`/bound on the generic dict read) | unbounded | **unbounded real** |
| `territory_ids`, `legal_authority`, `member_node_ids` | ORGANIZATION | `list[str]` | — | list-of-id |
| `doctrine_tags` | ORGANIZATION | `dict[DoctrineTag, float]` | **RESERVED-LINE** | doctrine content |
| `faction_balance` | ORGANIZATION (StateApparatus) | nested `FactionBalance` model: `finance_capital`/`security_state`/`settler_populist`/`stability`/`legitimacy` — all float, presumably `[0,1]`-simplex (not independently verified in this pass) | `[0,1]`-ish | nested composite |
| `state_budget` | ORGANIZATION (StateApparatus) | nested `StateBudget`: `revenue`/`available` (float), `allocated` (`dict[StateActionType, float]`), `imperial_rent_pool` (float) | unbounded | nested composite, money-semantic |
| `rng_seed` | ORGANIZATION (StateApparatus) | `int \| None` | — | **nondeterminism-critical**; `None` documented in-model as "a real non-determinism bug" |
| `is_institution` | ORGANIZATION | `bool` | `{T,F}` | boolean, deprecated field (`DeprecationWarning` on set) |
| `counter_intel_score` | ORGANIZATION | **not a declared Field on `Organization` or any subtype** (grep-confirmed zero hits) — a bare `org_data.get("counter_intel_score", 0.0)` dict read | undeclared | **undeclared attribute read** (distinct class from Territory's landmine: here there is no field to even exempt into `EXTRA_STAMPABLE_ATTRIBUTES`) |
| `momentum` | ORGANIZATION | **not a declared Field** (same as above); also **never written anywhere in `src/babylon`** (grep-confirmed) | undeclared | **undeclared, provably-always-default attribute** |
| `edge_type` (TRANSACTIONAL↔SOLIDARISTIC) | Relationship (org→community edge) | `EdgeType` (StrEnum) | closed set, **mutated in place post-creation** | **enum discriminant, runtime-mutable** — an edge changing its own type after creation is a distinctive shape (most systems treat `edge_type` as immutable-at-creation) |
| `value_flow` | Relationship (WAGES edge) | `float` (untyped generic read) | unbounded | **unbounded real** |
| `weight` | Relationship (REPRESSION edge) | `float` (untyped generic read via `.attributes`) | clamped `[0,1]` by the writer, no declared model bound | unit-interval by convention only |
| all `ooda.*`/`state_ai.*`/`reactionary.*` coefficients (≈70 fields across the three defines modules) | — | `float`/`int`/`bool`/`list[str]`/`dict[str,float]` | mostly `[0,1]` or bounded positive | coefficients |

**Enum-discriminant flag — same closed-vocabulary gap Territory found, four times over.** `OrgType`,
`ConsciousnessTendency`, `LegalStanding`, `JurisdictionLevel`, `DecisionMode` are all StrEnum
discriminants read by this system with no BSL `deffield enum` precedent for *this content's*
specific vocabularies yet (the query-lane note says enum fields are LANDED as a mechanism —
ADR195/ADR196 — but each concrete enum still needs its own `defenum` declaration and D-record
naming the member ordinal mapping; none exists for `OrgType`/`ConsciousnessTendency`/etc. today).

**Undeclared-attribute flag — a fresh finding, distinct from Territory's exemption pattern.**
`momentum` and `counter_intel_score` are read off organization nodes with **no corresponding
Pydantic field anywhere on `Organization` or its four subtypes** — not merely unseeded (like
`ooda_profile`, which IS a declared field, just never populated by a production scenario), but
structurally absent from the schema. A port that faithfully transcribes the read still needs a
declared-but-always-default `deffield`, or an explicit D-record naming why these two reads are
provably-`0.0` dead weight (same "provably uniform" `:const` treatment Territory's
`displacement_mode` got).

---

## 4. FLOAT-OP INVENTORY (execution order)

All arithmetic is binary64 Python `float`; **zero `exp`/`log`/`pow`/`sigmoid` calls anywhere in
the live orchestration path** (`ooda.py`, `layer0.py`, `layer3.py`, `initiative.py`,
`cycle_time.py`, `npc_stub.py`, `action_effects.py`, `decision.py`, `escalation.py` —
grep-confirmed). One `** 2` appears only in the **dead** `faction_dynamics.py:410`
(a population-variance helper, unreachable from `OODASystem`).

1. **Additive/multiplicative cycle-time composition** (`cycle_time.py:27-39`): four terms, one `max(..., floor)` clamp on `orient_time` — no hazard.
2. **Five-term weighted sum, initiative score** (`initiative.py:57`): `speed+institutional+counterintel+embeddedness+momentum_val` — plain sum, no hazard; `speed = weight * (1.0/cycle_time)` is a division whose denominator is provably `>0` (every `OODADefines` phase-time field is `gt=0`/floored).
3. **`resolve_action_order`'s sort key** (`initiative.py:79`): `(-score, org_id)` — negation + string tiebreak, deterministic.
4. **`compute_community_embeddedness`'s ratio** (`initiative.py:149`): `with_membership / len(member_ids)`, guarded `len(member_ids) > 0` before the divide; clamp `max(0.0, min(1.0, ...))` — nested-`if`-equivalent clamp, matches the Phase-1-style (`min`+`max` combined call) convention.
5. **`apply_autonomy_modifier`'s tradeoff formula** (`constraints.py:113-116`, **DEAD**): `1.0 - autonomy*scale*((n-1)/n)`, floored at `0.1` via `max(raw, 0.1)` — not on the live path, but a hazard if ever wired (division by `num_distinct_targets`, guarded `<=1` short-circuit above it).
6. **Faction-objective piecewise scoring** (`decision.py:106-219`): pure `if/elif` chains assigning bare-literal scores, one linear term each (`heat * const`); no hazard, but see §6 for the "hand-tuned literal table, not a formula" framing.
7. **Escalation-affinity formula** (`escalation.py:70-76`): `normalized_rank = rank/max_rank` (guarded `max_rank>0`); `affinity = max(0.0, 1.0 - |heat - normalized_rank|)`; `score = affinity * 2.0` — one division, one abs, one max — no libm transcendental, but the "Gaussian-like affinity" docstring language is misleading: this is a **triangular (V-shaped), not Gaussian, kernel** — a verbatim-transcribe-as-is naming oddity, not a math hazard.
8. **RNG tiebreak** (`decision.py:553`): `rng.uniform(0.0, 0.01)` — **not a float-op hazard in the libm sense, but a determinism hazard**: Python's `random.Random.uniform` is Mersenne-Twister-based and its exact bit-stream is a CPython implementation detail, not a portable cross-language contract the way IEEE-754 `+`/`*`/`/` are. Any BSL port needs its OWN declared RNG algorithm + byte-exact carrier-key derivation (bsl-language.rst §3.10 sketches `(session, tick, domain, stable_key)` but implements none of it — see §6).
9. **`_bump_repression_edge`'s clamp** (`action_effects.py:213,220`): `min(1.0, existing.weight + increment)` / `min(1.0, increment)` — **upper-only clamp**, no lower bound (increments are always non-negative by construction, so this is safe in practice, but it is NOT the same `max(lo,min(hi,x))` two-sided idiom Territory's `_write_clamped` uses) — a third clamp SHAPE in this estate, alongside Territory's two.
10. **`compute_consciousness_delta`'s multiplicative chain** (`action_effects.py:87-117`): `modifier*cadre_level*cohesion*effective_credibility*action_base`, then two conditional `*=` multipliers (EDUCATE contestation bonus, doctrine theory bonus — **RESERVED-LINE**), then a two-sided clamp `max(-cap, min(cap, x))` — the Phase-1-style two-sided clamp, a FOURTH clamp shape variant when combined with #9 and Territory's two.
11. **`_resolve_fascist_verb`'s wealth destruction** (`action_effects.py:351`): `current_wealth * (1.0 - pogrom_wealth_destruction)` — plain multiply, `wealth` itself has no declared upper bound (unbounded real, §3).
12. **`_propagate_heat`/`_propagate_infrastructure`** (`layer3.py:112,201`): `min(1.0, x+delta)` (upper-only) and `max(0.0, min(1.0, x+delta))` (two-sided) — the SAME two clamp shapes recur here too.
13. **Real→Int demotions:** none found in the live path — `int(org_attrs.get("employment_count", 0))`/`int(org_attrs.get("community_workforce", 1))` (`action_effects.py:570-571`) are casts of already-integer-typed reads, not truncating float demotions; `math.ceil(base_cost*modifier)` (`action_costs.py:76`, **DEAD**) is the one real Real→Int ceiling demotion in the whole package, unreachable from production.

**Clamp-shape census across the whole computation catalog: at least FOUR distinct implementations** —
(i) Territory's `_write_clamped` two-sided `max(lo,min(hi,x))` helper (not used here at all — OODA
never calls it); (ii) OODA's own hand-written two-sided `max(-cap,min(cap,x))` (#10, `_resolve_assimilate`
too); (iii) upper-only `min(1.0, x)` (#9, #12's heat path); (iv) `constraints.py`'s `max(raw, 0.1)`
single-sided floor (#5, dead). Port-as-is: transcribe each site's actual shape, do not unify them.

---

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 14.0** (`ooda.py:101`), confirmed against `_SYSTEM_CLASSES`
  (`simulation_engine.py:328-363`): 16th of 34 systems, immediately after `MetabolismSystem`
  (13.0) and before `FactionInfluenceSystem` (14.5) — all 13 Material Base systems plus
  `SubstrateSystem` (2.5) run before it this tick.
- **Reads from same-tick prior systems: structurally almost none.** OODASystem's real reads are
  almost entirely scoped to `NodeType.ORGANIZATION` (and derivatively TENANCY/SOLIDARITY/WAGES/
  REPRESSION edges) — no Material Base system (Vitality, Territory, Production, TickDynamics,
  ReserveArmy, Community, Lifecycle, Solidarity, ImperialRent, Dispossession, Decomposition,
  ControlRatio, Metabolism) writes an ORGANIZATION node. The one confirmed genuine cross-system
  read is `_resolve_fascist_verb`'s LOCKOUT branch: it reads `WAGES` edges' `value_flow`, which
  `ImperialRentSystem` (Material Base, earlier this tick, per `engine/systems/economic.py`'s
  4 `value_flow=` call sites) writes. `compute_community_embeddedness`'s TENANCY/
  `community_memberships` walk is a structural cross-system channel that is provably dead (§2.2).
- **Writes consumed later this tick / downstream ticks:**
  - `repression_faced` (SOCIAL_CLASS, from `_resolve_repressive`/`_resolve_fascist_verb`) — read
    downstream by `SurvivalSystem`'s P(S|R) denominator, `ConsciousnessSystem`'s continuous
    repression term (per `action_effects.py`'s own extensive docstrings), and grep-confirmed
    elsewhere as `ImperialRentSystem`/`OODASystem` in `tools/regression_scenarios.py:2899`'s
    channel-producer table.
  - `heat` (ORGANIZATION, from both `_resolve_repressive`/`_resolve_fascist_verb` and
    `layer3._propagate_heat` — **two independent writers of the same attribute in the same
    tick**, both keyed off the identical `repress_heat_delta`/`surveil_heat_delta` coefficients;
    not a conflict since they target different node/edge shapes, but worth naming as a doubled
    write path) — consumed next tick by `_gather_repress_target_candidates`'s own read (2.6),
    i.e. OODASystem partly reads its own prior-tick output.
  - `weight` (REPRESSION edge) — read by 3 downstream consumers per `action_effects.py`'s own
    docstring: `negotiate.py`, `bifurcation/axis.py`, `bifurcation/analysis.py`.
  - `edge_type` (TRANSACTIONAL→SOLIDARISTIC re-stamp) — dormant (ORGANIZE is one of the
    9 player-verb-registry verbs but is not currently one of them — MOVE/EDUCATE/RECRUIT/
    ATTACK_INFRASTRUCTURE/PROTEST/PROPAGANDIZE/PROVIDE_SERVICE/MAP_NETWORK/PROPOSE_ALLIANCE are
    the 9; ORGANIZE has no player resolver and no NPC material path either — this write is
    **provably unreachable in the current production system**, since ORGANIZE never resolves
    materially from any path).
  - `infrastructure` (community/territory) — consumed by whichever downstream systems read it
    (not traced in this pass; out of scope for a Phase-1 orchestration-level catalog).
- **Context/service usage with no BSL equivalent:**
  - `context.persistent_data["player_actions"]` (`ooda.py:177-178`) — the entire player-verb-
    registry gate; dormant on every canonical/headless scenario (§1).
  - `context.persistent_data["corridor_mesh"]` (`ooda.py:206`) — spec-108 opt-in, dormant while
    `TransportSystem` stays default-OFF (`tools/regression_scenarios.py:2729-2735`).
  - `context.persistent_data["turn_resolution"]` (write-only, `ooda.py:228`) — consumed by the
    web bridge (legacy, Amendment V) and any downstream reader of the JSON-mode dump; **not
    part of the hashed graph state**, so a BSL port need not reproduce it byte-for-byte, only
    its *material* graph-mutation side effects.
  - `services.defines.state_ai`/`.organization`/`.reactionary`/`.politics`/`.transport` — five
    distinct defines sub-models threaded through one system's `step()`, more than any other
    system inventoried so far in this port-estate survey.
- **DORMANCY on canonical scenarios — the central finding of this inventory.**
  `tools/regression_scenarios.py:2750-2760` (`COVERAGE_GAPS_DATA`) states verbatim: *"no
  organizations are seeded in any canonical scenario (single_county, Task 8, seeds only
  2 SocialClass + 1 Territory — no ORGANIZATION nodes); the per-tick ORGANIZATIONAL_ACTION
  summary event fires every tick but with org_count=action_count=layer0_count=0 — the
  turn-resolution loop's own control flow runs, but no organizational action, initiative
  resolution, or verb-resolver logic ever exercises."* The same file (`:2011-2021`) records that
  even `detroit_tri_county` — the real, committed e2e headless-runner baseline, **not** one of
  the qa:regression canonical scenarios but a genuine live-run artifact — was tried as evidence
  and **rejected**: its `organizational_action` event payload is `{org_count:0, action_count:0,
  layer0_count:0}` on all 5 ticks of the committed baseline. Grep-confirmed: neither
  `engine/scenarios/_legacy.py` (1,270 lines) nor `_legacy_wayne.py` (764 lines, except its own
  two hardcoded orgs at lines 577-662) stamps an `OrgType.` literal.
  The **only two factories that ever seed a real `NodeType.ORGANIZATION` node** are
  `_legacy_wayne.create_wayne_county_scenario` (the "live Wayne campaign" referenced throughout
  the `ooda/` docstrings — a `CivilSocietyOrg` "ORG001" + a `StateApparatus` "Detroit Police
  Department" with `faction_balance` + `rng_seed=0`, `_legacy_wayne.py:577-662`) and the new
  `org_probe` scenario (`engine/scenarios/org_probe.py`, registered in
  `tools/regression_scenarios.py`'s `SCENARIOS` dict at line 128, dated 2026-08-11/12 — one
  `CivilSocietyOrg` + one `StateApparatus`, both **without** `territory_ids` or `ooda_profile`,
  the latter WITH `faction_balance`/`rng_seed=0`). `org_probe`'s own docstring and its
  `SCENARIO_COVERAGE_DATA` entry (`tools/regression_scenarios.py:2596-2615`) explicitly
  **disclaim** exercising OODASystem's business logic: *"Deliberately minimal: it exists to
  prove NodeType.ORGANIZATION nodes are real, non-fixture graph shape (Task 11), not to
  exercise System business logic — no SystemEvidence rows are claimed here."* So even as of
  this inventory's date, **no conformance oracle for OODASystem's live behavior exists on any
  scenario the byte-identical `qa:regression` gate runs** — a port's fixtures must be entirely
  hand-built, more so than any other system inventoried in this port-estate survey to date
  (Territory's dormancy was partial — Phase 1 heat was live; OODASystem's is total).

---

## 6. BLOCKER ASSESSMENT — proposed TRAIN SPLIT

Given in §1's numbers, roughly 42% of the `ooda/` package by line count has zero production
caller, and the live path itself splits cleanly into independently-portable concerns. This
inventory recommends **five trains**, not one pack:

### Train A — Turn skeleton + initiative scoring (small, mostly query-lane-ready)

| Computation | Verdict | Detail |
|---|---|---|
| Layer 0 auto-EMPLOY stamp (§2.1) | **PORTABLE NOW** | Pure node-type filter + a fixed event tag, no arithmetic, no writes. Trivial `for-each`/`exists` over ORGANIZATION nodes filtered by `org_type`. |
| Cycle-time computation (§2.2, `cycle_time.py`) | **PORTABLE NOW** | Four-term additive/multiplicative formula, all `[0,1]`-ish domain coefficients, a 4-way `decision_mode` dict lookup — needs the LANDED enum-field mechanism (ADR195/196) for `decision_mode`, with its own `defenum`/D-record (no existing precedent for this specific enum — see §3). |
| Initiative score + sort (§2.2) | **PORTABLE WITH D-RECORD** | The five-term weighted sum and `(-score, id)` sort are trivial. `counter_intel_score` and `momentum` are undeclared-attribute reads (§3) — D-record as `:const 0.0` (Metabolism-D-2/Territory-`displacement_mode`-style "provably uniform"), same class, needs its own naming since these have no declared field at all (not just no seeder). |
| `compute_community_embeddedness` (§2.2) | **PORTABLE, but structurally dead** | The TENANCY-walk + `community_memberships` share formula is `fold`/typed-`neighbors`-shaped and evaluable under the LANDED query lane, but is provably `0.0` on every real game (no `CommunitySystem` producer) — a D-record confirming it stays `:const 0.0` is cheaper than porting live traversal logic for a value nothing ever changes. |

### Train B — NPC static-priority-queue dispatch (small, but must transcribe the inert-verb defect)

| Computation | Verdict | Detail |
|---|---|---|
| `_NPC_PRIORITIES` selection + `check_eligibility` (§2.4) | **PORTABLE NOW** | Fixed lookup tables + a linear eligibility scan, all closed-vocabulary. `ELIGIBILITY_MAP`'s 4×26 matrix needs `defenum`s for `OrgType`/`ActionType` (both already closed StrEnums with precedent from other landed packs' `NodeType`/`EdgeType` handling, though these two specific enums have none yet). |
| The blind-no-op materialization for non-REPRESS/SURVEIL NPC actions (§2.4) | **PORTABLE NOW, D-record the defect** | This is the SIMPLEST computation in the whole system — `ActionResult(success=True, events_generated=[ORGANIZATIONAL_ACTION])`, no graph write at all — but it is also the single most consequential fact about OODASystem's live behavior (§5) and MUST be D-recorded explicitly as "port-as-is: NPC-selected EDUCATE/ORGANIZE/AGITATE/RECRUIT/FUNDRAISE/PROVIDE_SERVICE/BUILD_INFRASTRUCTURE/EMPLOY/DENOUNCE are cosmetic, not a simplification of the port." |

### Train C — RuleBasedStateAI decision engine (BLOCKED on the RNG kernel intrinsic)

| Computation | Verdict | Detail |
|---|---|---|
| Candidate generation, faction-objective scoring, escalation scoring (§2.5) | **PORTABLE NOW (arithmetic only)** | Pure `if/elif` literal tables and a two-term weighted-sum-plus-affinity formula — no libm, no query-lane needs beyond reading `faction_balance`/`state_budget`/`heat` fields. `_VERB_COSTS`/`_LEGITIMACY_COSTS` (hardcoded dicts bypassing `defines.yaml`, §2.5) is itself a D-record-worthy oddity to transcribe, not silently move into `defines.yaml`. |
| The RNG tiebreak (`rng.uniform(0.0, 0.01)`, §2.5/§4) | **BLOCKED — RNG kernel intrinsic not implemented** | bsl-language.rst §2.8/§3.10/chapter C13 SPEC a `(session, tick, domain, stable_key)`-carrier RNG kernel intrinsic ("draft ruling — Phase 1 review"), but zero grep hits for RNG in `rust/crates/babylon-bsl/src/*.rs` or `rust/crates/babylon-tick/src/*.rs` confirm it is unimplemented. Name the exact missing lane: **the RNG kernel intrinsic chapter (C13)**. Until it lands, the tiebreak cannot be expressed deterministically-and-portably; dropping it silently would also drop the `rng_seed=None` non-determinism bug the frozen model itself names — port-as-is requires either the real intrinsic or an explicit "tiebreak omitted, D-recorded" deviation, never a silent fixed constant standing in for randomness. |
| Sparrow topological targeting (§2.7) | **BLOCKED — query-lane depth + dead in practice** | Needs a full centrality/betweenness/articulation-point graph algorithm over a dynamically-built subgraph — nothing like this exists in the landed query lane (fold/select-max/neighbors are single-hop; rustworkx centrality is a whole-graph fixed-point computation). Also structurally dead today (empty SOLIDARITY subgraph, §2.7) — **DEFER, not urgent**, since the fallback (`heat*visibility` sort) is what actually runs. |
| `select_repress_target` sans topology (§2.5) | **PORTABLE WITH D-RECORD** | A `select-max`-shaped reduction over `(id,heat)` pairs with an ascending-id tiebreak — matches the LANDED §2.7 tiebreak mechanism exactly (the same idiom Territory's `_find_sink_node` proved out, ADR197). Needs `select-max` over a computed candidate set built from a `for-each`/`fold`-style filter, which the landed query lane serves. |

### Train D — Action effects (BLOCKED on edge-attribute storage)

| Computation | Verdict | Detail |
|---|---|---|
| `compute_consciousness_delta` (five-factor formula, §2.9) | **PORTABLE NOW (arithmetic)**, gated by **RESERVED-LINE** doctrine content | The multiplicative chain + two-sided clamp is expressible with landed BSL primitives (Currency/Real ops, exp/log/floor intrinsics all unneeded here — no transcendentals). The doctrine-tag theory bonus reads `DoctrineTag.CLASS_ANALYSIS` — RESERVED-LINE content, describe-never-propose per this inventory's mandate; port scope must explicitly carve it out or escalate to the Director. |
| `_resolve_agitate`, `_resolve_assimilate` (§2.9) | **PORTABLE NOW** | Pure arithmetic, no edge involvement. |
| `_resolve_repressive`'s `repression_faced` bump (node write) | **PORTABLE NOW** | A `update-node` against a resolved `NodeRef` with a two-sided-clamp-equivalent nested `if` — matches landed precedent exactly. |
| `_bump_repression_edge` (REPRESSION edge create-or-strengthen, §2.9) | **BLOCKED — edge-attribute storage** | `GraphSubstrate` stores exactly one `f64` "strength" per `(type, from, to)` triple (`structural_verbs.rs:16-26`); `add-edge` is landed for the CREATE case (`:strength` field only — `structural_verbs.rs:877-910`), but the STRENGTHEN-existing-edge case needs `update-edge`, which **refuses by name** at `structural_verbs.rs:693-700` ("has no substrate storage... widening that state widens the canonical state_hash field set, which is a declared Phase-2/substrate decision"). Also needs an `EdgeRef` field READ (`existing.weight`) to decide create-vs-strengthen — Slice 2, not built. Name the exact missing lane: **`update-edge` storage widening (R9 chapters C2/C12, D35/D65) + Slice 2 `EdgeRef` field reads.** |
| `_propagate_repression_to_class_base` (org→class SOLIDARITY fan-out, §2.9) | **BLOCKED — query evaluation depth** | Needs a `fold`/`for-each` over SOLIDARITY-typed incoming edges filtered by source-type, id-sorted, with an even-split write to each — the landed query lane's `fold`/`for-each`/typed-`neighbors` primitives cover the traversal shape (favorable structural match, like Territory's spillover), but the WRITE half needs `update-node` against each computed member — mechanically portable once the traversal is proven out; flagged BLOCKED here only pending that proof, not a deep gap. |
| `_resolve_fascist_verb`'s LOCKOUT WAGES-edge attenuation (§2.9) | **BLOCKED — edge-attribute storage** | `value_flow` is a declared edge attribute distinct from the single `f64` strength `GraphSubstrate` stores — same missing lane as the REPRESSION-edge case above, doubly so (this needs BOTH a read of an existing WAGES edge's `value_flow` AND a multiplicative in-place update, neither served). |
| POGROM's wealth destruction (§2.9) | **PORTABLE NOW** | A `Currency`-typed node-attribute multiply-and-write — same `bare-scaled-Int` workaround class as Territory's `rent_level`/Metabolism's `entropy_factor` (ADR183 declared-deviation), since `wealth` is `:field`-sourced → `Value::Real`, not `Value::Currency`. |

### Train E — Layer 3 propagation (mixed: heat/infra PORTABLE, edge-retyping BLOCKED)

| Computation | Verdict | Detail |
|---|---|---|
| `_propagate_heat` (§2.11) | **PORTABLE NOW** | Single `update-node` per matching action result, upper-only clamp — trivial. |
| `_propagate_infrastructure` (§2.11) | **PORTABLE NOW** | Same shape, two-sided clamp — trivial; the corridor-mesh splash half is dormant (TransportSystem default-OFF) and out of scope until that system's own port train lands. |
| `_propagate_edge_transitions` (TRANSACTIONAL→SOLIDARISTIC, §2.11) | **BLOCKED — edge retype, unreachable in production anyway** | Mutating an edge's OWN TYPE post-creation (`graph.edges[org,target]["edge_type"] = SOLIDARISTIC`) is not an attribute write at all in the BSL substrate model — `GraphSubstrate` keys edges BY `(type, from, to)`, so "retyping" an edge means removing the old triple and adding a new one, a shape no landed verb expresses directly. Doubly moot: §5 shows this write is **provably unreachable** in the current production system (ORGANIZE has no material resolver on any path) — **DEFER, do not spend the train's budget here.** |

### Modules explicitly OUT OF SCOPE for any near-term port train

The 2,568 dead-per-grep lines cataloged in §1 (attention thread-manager/observation, all six
`state_ai/*_effects.py` sub-verb resolvers, `constraints.py`, `action_costs.py`,
`lifecycle_capacity.py`) should **not** be ported until the Director rules on whether to wire
them into the live Python path first (per the standing "port-as-is, not port-as-designed" law —
porting unreached code invents a BSL behavior the frozen engine never actually produces) or to
retire them. The player-verb registry (`engine/actions/`, 1,814 lines) is a separate, comparably-
sized estate that deserves its own Phase-1 inventory rather than folding into this one.

**ADR184's forward capacity model is a design target, not a transcription source.** The Rust
`Capacity`/`Candidate`/`allocate` unification (owned by an `Organization`, repression and
revolutionary action ranked by the same allocator) is the STATED end-state, but it has "ZERO
external callers" today (ADR184's own words) and is structurally unlike anything the frozen
Python `OODASystem` does (which runs two disconnected budget systems: `OODAProfile.action_points`
— itself unenforced, §1 — and `StateBudget`, used only by `RuleBasedStateAI`). A port that tried
to retrofit ADR184's unified allocator onto Trains B/C would be porting the FUTURE architecture,
not the frozen system — port-as-is law says transcribe the two disconnected budgets faithfully,
with a D-record naming ADR184 as the eventual convergence point, not the source of truth today.

---

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/ooda/test_ooda_system.py` | 492 | **Primary conformance-oracle candidate** for the orchestrator itself — `step()`-level behavior. |
| `tests/unit/ooda/test_action_effects.py` | 988 | Exhaustive per-verb unit coverage of `action_effects.py`'s live switch — the largest single conformance-oracle candidate in the estate. |
| `tests/unit/ooda/test_npc_stub.py` | 298 | NPC dispatch (static queue + state-AI gate) coverage. |
| `tests/unit/ooda/test_initiative.py` | 232 | Initiative-score component coverage. |
| `tests/unit/ooda/test_layer3.py` | 287 | Layer-3 propagation coverage, incl. corridor-mesh splash. |
| `tests/unit/ooda/test_types.py` | 295 | Pydantic model validation — schema-level, not tick-behavior. |
| `tests/unit/ooda/test_eligibility.py` | 267 | `ELIGIBILITY_MAP`/`check_eligibility` — closed-vocabulary conformance candidate. |
| `tests/unit/ooda/test_reactionary_ooda_verbs.py` | 163 | POGROM/LOCKOUT/VIGILANTISM coverage. |
| `tests/unit/ooda/test_cycle_time.py` | 128 | Cycle-time formula coverage. |
| `tests/unit/ooda/test_layer0.py` | 93 | Layer-0 stub coverage. |
| `tests/unit/ooda/test_defines.py`, `test_coefficient_derivations.py` | 168, 253 | `OODADefines` bounds + `validate_derivations` cross-check coverage — schema/derivation, not behavior. |
| `tests/unit/ooda/test_membership_overlap_canonicalization.py` | 84 | `_compute_membership_overlap` coverage. |
| `tests/unit/ooda/test_action_costs.py` | 315 | **DEAD-code coverage** — `compute_action_cost` has zero production callers (§1); this is a well-specified behavioral contract for code the tick path never reaches. |
| `tests/unit/ooda/test_constraints.py` | 159 | **DEAD-code coverage** — same caveat, `constraints.py`. |
| `tests/unit/ooda/test_lifecycle_capacity.py` | 170 | **DEAD-code coverage** — same caveat, `lifecycle_capacity.py`. |
| `tests/unit/engine/laws/test_law_ooda.py` | 196 | **Property-based invariant contracts** — the behavioral-contract layer the port's own conformance scenarios should re-prove independent of bit-exactness (same role as Territory's `test_law_territory_system.py`). |
| `tests/unit/institution/test_ooda_effects.py` | 104 | Institution-estate cross-check on OODA effects — narrower scope, worth reading before excluding. |
| `tests/unit/state_ai/*.py` (18 files) | 6,353 total | **Overwhelmingly DEAD-code coverage** — `test_administer_effects.py` (227), `test_co_opt_effects.py` (267), `test_repress_effects.py` (617), `test_territory_effects.py` (936), `test_legislate_effects.py` (124), `test_faction_dynamics.py` (1,255), `test_attention_threads.py`/`test_thread_lifecycle.py` (151+158) all exercise modules §1 confirms have zero production callers. `test_decision_targeting.py` (209), `test_escalation.py` (552), `test_sparrow.py`/`test_sparrow_targeting.py` (135+517), `test_faction_balance.py` (161), `test_state_budget.py` (118), `test_state_enums.py` (263), `test_observability.py` (219, dead module), `test_defines.py` (63) cover the LIVE `decision.py`/`escalation.py`/`sparrow.py` seam and are genuine conformance-oracle candidates. |
| `tests/contract/state_ai/*.py` (7 files) | 2,815 total | **Contract-level, overwhelmingly on DEAD modules** — `test_administer_contract.py` (161), `test_co_opt_contract.py` (216), `test_repress_contract.py` (284), `test_territory_contract.py` (691), `test_legislate_contract.py` (121) contract the six dead effects modules. `test_decision_contract.py` (511) and `test_faction_contract.py` (695) — the latter almost certainly covers `renormalize_faction_balance`, which IS live but called from `electoral.py`, a different System — cover live-elsewhere code. `test_thread_contract.py` (112) contracts the dead attention-thread estate. |
| `tests/integration/test_ooda_detroit.py` | 431 | Integration coverage against a Detroit-shaped fixture — worth reading fully before scoping the port's fixture design. |
| `tests/integration/test_state_ai_integration.py` | 709 | Integration coverage of `RuleBasedStateAI` end-to-end. |
| `tests/integration/test_state_ai_wayne_county.py` | 245 | Integration coverage against the ONE scenario that seeds real orgs. |

**qa:regression byte-gate coverage.** Per §5, `tools/regression_scenarios.py`'s own
`COVERAGE_GAPS_DATA` declares OODASystem's business logic entirely absent from every canonical
scenario's evidence — the byte-identical hash gate (`tools/regression_test.py::
graph_content_hash`) would in principle catch a regression, but since `org_count` is
provably `0` on every canonical run, **no canonical scenario exercises anything this inventory
catalogs**. A port's conformance fixtures must be hand-built `.bscn` scenarios seeding real
Organization nodes (`org_probe`'s field-value choices are a ready-made template, since its own
docstring cites mirroring `_legacy_wayne`'s constants exactly), analogous to Metabolism's/
Territory's hand-built conformance packs, but with materially larger scope given this system's
26-verb, 2-org-type-dispatch surface.

---

**Ideological/RESERVED-LINE surface named in this inventory (describe-only, per mandate):**
the Doctrine Tree `CLASS_ANALYSIS` theory bonus (§2.9, `action_effects.py:99-111`) and the
`StateFaction`-to-`PolicyAxis` proxy mapping in LEGISLATE routing (§2.10/§2.8,
`npc_stub.py:476-480`: FINANCE_CAPITAL→war_posture, SECURITY_STATE→police_budget,
SETTLER_POPULIST→border_regime — a "declared U9 proxy" the codebase's own comment marks as
provisional pending U10's governing-platform replacement). Neither is proposed for change here.

---

## Adjudication (2026-08-12)

Adjudicated against the **current dev tree** (`9324482f`) by a second read-only pass, with fresh
`rg`/`Read` against every anchor cited below. The inventory's dead-code census, tick position,
clamp-shape census and cross-system channel table hold up; its **central dormancy premise does
not**, and two blocker rows name the wrong (or an incomplete) lane.

1. **CORRECTION — the package is `src/babylon/ooda/`, not `ai/ooda/`.** The executive summary's
   first sentence points at a path that does not exist (`ls ai/ooda` → "No such file or
   directory"). The real estate is 28 `.py` files / 6,112 lines under `src/babylon/ooda/`
   (`find src/babylon/ooda -name '*.py' | wc -l`; `-exec wc -l {} +` tail). The line count and
   the ~42% dead-weight ratio are right; only the path is wrong. Cosmetic, but the summary is
   the part a Director reads.

2. **CORRECTION — OODASystem is NOT dormant on every canonical scenario. `org_probe` is
   canonical, byte-gated, and drives the live state-AI decision seam every tick.** The
   inventory's own §1/§5 record that `org_probe` seeds two Organization nodes, then §5 adopts
   `COVERAGE_GAPS_DATA`'s "no organizations are seeded in any canonical scenario"
   (`tools/regression_scenarios.py:2751-2760`) as the headline and concludes "OODASystem's is
   total." That gap row is **stale against its own file**: `SCENARIOS` (the registry
   `qa:regression compare` iterates — `tools/regression_test.py:1424`, `for name in SCENARIOS`)
   carries `"org_probe"` at `tools/regression_scenarios.py:128`, `PENDING_CEREMONY` is an empty
   frozenset (`:143`), and both baselines are committed (`tests/baselines/org_probe.json`,
   `tests/baselines/dense/org_probe.csv`). `.mise.toml:974` names `org_probe` as one of the two
   CI-lane vault scenarios. Materially: `create_org_probe_scenario` seeds a `StateApparatus`
   with `faction_balance=state_apparatus_balance` and `rng_seed=0`
   (`src/babylon/engine/scenarios/org_probe.py:132-146`) — which is **exactly** the gate
   condition `_try_state_ai_dispatch` tests (`npc_stub.py:374-377`, "`if faction_data is None:
   return None`"). `RuleBasedStateAI.select_action` and its `rng.uniform(0.0, 0.01)` tiebreak
   therefore run on a canonical, baselined scenario every tick. The sibling Doctrine inventory
   caught this same staleness independently and verified it three ways; this inventory reported
   both halves of the contradiction and resolved it the wrong way.

3. **CORRECTION — the RNG kernel half is BUILT; only the BSL-side binding is missing.** §6 Train
   C states "zero grep hits for RNG in `rust/crates/babylon-bsl/src/*.rs` or
   `rust/crates/babylon-tick/src/*.rs` confirm it is unimplemented" and files the blocker as "the
   RNG kernel intrinsic (unbuilt)". The grep scope missed the crate that has it:
   `rust/crates/babylon-kernel/src/rng.rs` implements `KernelRng::for_carrier(session_id, tick,
   domain, stable_key)` over a pinned `ChaCha8Rng` (`:69-78`), with the §3.10 C13 carrier key
   derived as `SHA256(session ‖ tick_le8 ‖ salt_le8 ‖ len-framed domain ‖ len-framed stable_key)`
   (`:53-63`) and a bit-deterministic `next_f64` (`:92-95`); `rand_chacha`/`rand_core` are
   declared dependencies at `rust/crates/babylon-kernel/Cargo.toml:12-13`. It has **zero external
   callers** (`rg 'KernelRng|seed_for' rust/crates --type rust` outside its own module returns
   only the `lib.rs` re-export). The honest statement of the gap is therefore narrower and
   sharper than "unbuilt": the algorithm, the key convention and the salt are landed; what is
   missing is a name in `DECLARABLE_INTRINSICS` (`declarations.rs:110` — `["exp","log","floor"]`)
   and a dispatch arm in `KernelIntrinsicHost` (`intrinsic_host.rs:59-70`). Consequence for the
   verdict: Train C's blocker is the *same category* as Survival's `exp` gap, not a separate
   architectural class.

4. **CORRECTION — Train C's "port-as-is requires either the real intrinsic or an explicit
   'tiebreak omitted' deviation" misses the third, already-ruled option.** `rng.rs:30-33` states
   the ruling verbatim: "**Streams differ from Python by design (R8):** this is the pinned
   Rust-side replacement, not a port. Python's MT19937 streams are a closed epoch; stochastic
   baselines re-bless at cutover under ensemble-envelope comparison, not byte replay." A port of
   the tiebreak is therefore never a byte-identical transcription target under any lane, and the
   pack's conformance vector for it must be an ensemble envelope. (The Doctrine inventory records
   the corresponding `reports/p27-tolerance-and-envelope-derivations.md` §4.1 methodology row;
   this inventory does not cite it.)

5. **CORRECTION — Train D's "`add-edge` is landed for the CREATE case" is false in rule
   position, and the same gate widens Train E.** `_bump_repression_edge`'s create branch is graded
   as served by `structural_verbs.rs:877-910`. That is the verb's *implementation*; a **rule**
   containing it never loads. `DEFERRED_SHAPE_VERBS = ["add-node","remove-node","add-edge",
   "remove-edge","add-hyperedge","remove-hyperedge"]` (`structural_verbs.rs:1352-1359`) is
   refused **at content load** by `check_no_deferred_shape_verbs` (`:1388-1406`), wired
   unconditionally into `rule_pipeline::load_rule_form` at `rule_pipeline.rs:268`
   (`.map_err(LoadError::DeferredShapeVerb)`), because Task 12's collect-then-apply pre-state
   split "does not serve the graph-shape verbs, only update-node/emit/guard/for-each"
   (`structural_verbs.rs:691-694`). So Train D's REPRESSION-edge row is blocked on **two**
   independent lanes (this load gate for create, `update-edge` substrate storage for strengthen,
   Slice 2 for the read), and Train E's `_propagate_edge_transitions` row is blocked by
   construction rather than by "no landed verb expresses it directly" — both halves of a retype
   are refused at load. Name the missing lane by name: **the Task-12 deferred-shape-verb load gate
   (placeholder-id design, `EffectExecutor::collect_effects`'s own escalation)**.

6. **CONFIRMATION — the dead-per-grep census is sound.** `constraints.py`, `action_costs.py` and
   `lifecycle_capacity.py` have zero callers outside `tests/` (only `_helpers.py`'s docstring
   mentions `action_costs` by name). Every `state_ai/*_effects.py` module's only importers are
   other members of the same dead cluster plus the `state_ai/__init__.py` re-export hub
   (`rg -c 'state_ai\.<mod>|from .<mod>' src/`). The 42%-dead framing stands.

7. **CONFIRMATION — tick position and the `repression_faced` channel.** `position: ClassVar[float]
   = 14.0` (`ooda.py:101`); `_SYSTEM_CLASSES` (`simulation_engine.py:328-364`) places it 16th of
   34, after `MetabolismSystem` and before `FactionInfluenceSystem` (whose own `position` I read
   as 14.5 at `faction_influence.py:53`). OODA writes `repression_faced` at
   `action_effects.py:306,346,473`; `SurvivalSystem` @15.0 reads it at `survival.py:130` and
   `StruggleSystem` @16.0 at `struggle.py:336` — same tick, downstream, as claimed. **One
   refinement:** OODA is not the sole producer — `ImperialRentSystem` @9.0 also writes it
   (`economic.py:640`), so the WS1 channel row must name two producers, not one.

8. **CONFIRMATION — `update-edge` refusal and the single-`f64` edge state.** `structural_verbs.rs`
   module doc `:16-26` and the refusal arm at `:709-710` are exactly as quoted: `GraphSubstrate`'s
   edge state is one `f64` strength keyed by `(type, from, to)`, hyperedges carry no attributes,
   and widening either widens the canonical `state_hash` field set (III.7). Slice-2/Slice-3/Slice-4
   heads are enumerated at `evaluator.rs:503-512`; `SERVED_QUERY_HEADS` is exactly
   `["nodes","neighbors"]` (`:527`).

9. **CONFIRMATION — the RESERVED-LINE flags are correctly placed and complete for this system.**
   The Doctrine-Tree `CLASS_ANALYSIS` theory bonus (`action_effects.py:99-111`) and the
   `StateFaction`→`PolicyAxis` LEGISLATE proxy (`npc_stub.py:476-480`) are both genuinely
   Director-reserved, and both are described rather than proposed upon, per mandate. No further
   ideological surface was found unflagged in the live path.

**FINAL VERDICT: BLOCKED as one pack — DEFER to a train split, but re-cut it, and retract the
dormancy premise.** The five-train shape is sound in outline; the grading is not. Trains D and E
are blocked by a lane the inventory never names (the Task-12 deferred-shape-verb **load** gate,
`rule_pipeline.rs:268`), on top of `update-edge` and Slice 2; Train C's blocker is narrower than
claimed (the kernel RNG is landed at `babylon-kernel/src/rng.rs`; only the intrinsic binding is
missing) and its conformance target is ruled to be an ensemble envelope, never byte replay.
Crucially, the "no conformance oracle exists on any canonical scenario" conclusion is **withdrawn**:
`org_probe` is canonical, baselined, CI-gated, and its `StateApparatus` seeds `faction_balance` +
`rng_seed=0`, so the live state-AI decision seam — the single most consequential surface in this
system — runs on the byte gate today.

**INADEQUATE-COVERAGE NOTE.** A re-read must add, at minimum: (i) a live-behaviour trace of
`org_probe` through `_try_state_ai_dispatch` → `RuleBasedStateAI.select_action` →
`_gather_repress_target_candidates` (whose `heat > 0` filter meets a `CivilSocietyOrg` seeded
`heat=0.0`, `org_probe.py:112-123`) → the REPRESS re-stamp → `_resolve_repressive`, stating
exactly which graph writes survive the `WorldState↔graph` round-trip into `graph_content_hash`
(`tools/regression_test.py:924-964` hashes the **projection**, not the live mid-tick graph — the
same round-trip-loss caveat the sibling Survival inventory documents for `p_acquiescence` on
ORGANIZATION nodes applies here and is not analysed); (ii) `check_no_deferred_shape_verbs` as a
first-class blocker class applied to every `add-*`/`remove-*` row in Trains D and E; (iii) a
`rg` over `rust/crates/babylon-kernel/` before any future "unbuilt" claim about kernel services.
