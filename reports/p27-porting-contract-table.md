# Program 27 — Porting Contract Table (Phase 0 scaffold)

**Status:** Phase 0 scaffold — created 2026-07-29. Closed in Phase 3 (spec §8.4:
"the single named artifact — every earlier 'port checklist' reference means
this"). The **Sign-off** column is intentionally empty; it fills in as each
system's Rust port lands a reviewed, mutation-validated law suite.

**Source of truth for system membership/order:**
`src/babylon/engine/simulation_engine.py:328-363` (`_SYSTEM_CLASSES`, 34
entries — the CLAUDE.md "33" is stale by one, per spec §6.4).

**Classification counts (spec §6.4):** BSL_RULES 17 · HYBRID 12 ·
RUST_INTRINSIC 5 = 34.

**Coverage counts (live gate-coverage sentinel, cross-checked mechanically
against `tools/regression_scenarios.py::SCENARIO_COVERAGE_DATA` /
`COVERAGE_GAPS_DATA` for this table):** 17 systems evidenced by canonical
scenarios, 17 originally declared `CoverageGap` rows, 0 blind. The freeze-tag
floor (Task 17) is **all 34** — a `CoverageGap` row alone does not survive the
freeze; Task 11 backfills each gapped system with a property law or
transcribed unit suite, or a Director-signed waiver.

**Task 11 status (2026-07-29): CLOSED — 17/17 gapped systems now carry a
property-law suite** under `tests/unit/engine/laws/test_law_<system>.py`
(one file per system, hypothesis-based, `mise run test:q -- tests/unit/engine/laws/`
green). Zero waiver rows were needed — every gapped system yielded at least
one well-grounded, source-evidenced law. The coverage-instrument column below
now reads `law:tests/unit/engine/laws/test_law_<x>.py` for all 17 in place of
`GAPPED — Task 11`.

## Method

- **Classification / float-hazard inventory:** transcribed verbatim from spec
  §6.4 (`docs/superpowers/specs/2026-07-28-program-27-refoundation-design.md`).
- **LOC:** `wc -l` on each system's home file (`src/babylon/engine/systems/*.py`
  for all but the three domain-hosted systems below).
- **RNG usage:** mechanical grep for `resolve_rng(` call sites (not merely the
  import — every system imports `kernel.system_base`, which *defines*
  `resolve_rng`, so import-presence alone is a false positive). Five systems
  actually call it: FactionInfluence, FascistFaction (`reactionary.py`),
  Electoral, Doctrine, Struggle. A sixth — OODASystem — threads RNG
  transitively through `ooda/npc_stub.py::select_npc_actions` into
  `ooda/state_ai/{decision,repress_effects,administer_effects}.py`
  (`random.Random(rng_seed)` at each), confirmed by direct read, not grep
  alone.
- **`round()` sites — in-tick vs presentational:** direct file scan plus one
  import-hop into each system's own imported formula/domain modules, then each
  hit was **opened and read** to separate real production rounding from
  `>>> round(...)` doctest examples (the doctest ones are presentational only
  — they exist to pin a formula's documented example output, not to touch tick
  state). This is a **1-hop trace, not an exhaustive transitive closure** —
  deeper multi-hop round() sites (e.g. `domain/economics/circulation/turnover.py`,
  reachable from `ImperialRentSystem` only through the optional
  `_invoke_n_if_wired` → `engine/systems/n.py` (`Vol2CirculationStep`) gated
  sub-stage) are flagged, not fully traced, and are a candidate for the Task 17
  pre-freeze deep audit rather than re-litigated here.
- **Coverage instrument:** mechanically extracted from
  `tools/regression_scenarios.py::SCENARIO_COVERAGE_DATA` (scenario names) and
  `COVERAGE_GAPS_DATA` (gap reason, truncated) via `ast.literal_eval` over the
  parsed module (the sentinel `src/babylon/sentinels/gate_coverage/checks.py`
  parses the same literal).

## The 34-row table

| # | System | Classification | LOC | Float hazards (spec §6.4) | RNG usage | `round()` sites (in-tick / presentational) | Coverage instrument | BSL-vs-intrinsic split | Sign-off |
|---|---|---|---|---|---|---|---|---|---|
| 1 | VitalitySystem | BSL_RULES | 255 | none named in spec §6.4 inventory | none found | none found | `glut`, `imperial_circuit`, `starvation`, `two_node` | pure BSL rules, no named intrinsic | |
| 2 | TerritorySystem | BSL_RULES | 370 | none named in spec §6.4 inventory | none found | none found | law:tests/unit/engine/laws/test_law_territory_system.py (no PENAL_COLONY/CONCENTRATION_CAMP territory types seeded; heat/eviction/necropolitics never engage; the one write (heat) is territory-scoped, unobservable via `entity_delta`; Task 11 property laws cover the heat clamp, population non-negativity, eviction-flag monotonicity, and inactivity on ungated non-territory nodes) | pure BSL rules, no named intrinsic | |
| 3 | SubstrateSystem | HYBRID | 338 | none named in spec §6.4 inventory (numeric core: metabolic-rift lattice via `formulas.metabolic_rift`) | none found | none found (imports `formulas.metabolic_rift`, `domain.dialectics.instances.{levels,scale}`, `domain.economics.tick.graph_bridge` — no `round()` hits at 1 hop) | law:tests/unit/engine/laws/test_law_substrate.py (no canonical scenario seeds HEX-type nodes; stock pass-through never touches a node; Task 11 property laws cover the clamp, monotone depletion under the shipped regeneration_rate=0.0 default, nation-level extensive-sum conservation, and inactivity) | BSL guards/thresholds + named intrinsic core (metabolic-rift lattice) | |
| 4 | ProductionSystem | HYBRID | 268 | none named in spec §6.4 inventory (numeric core: `domain.economics.tensor` Leontief-style lookups) | none found | none found at 1 hop (`domain.economics.tensor` has no `round()`) | `imperial_circuit` | BSL guards/thresholds + named intrinsic core (tensor lookups) | |
| 5 | TickDynamicsSystem | RUST_INTRINSIC | 2,558 (`domain/economics/tick/system/__init__.py`) | **sigmoids** (spec §6.4) | none found via `resolve_rng`; not traced further (2,558-line Vol I/II/III tensor core, out of 1-hop scope) | **9 in-tick** (7 own file, lines 1164-1167 + 2336-2340, all dict-construction rounding to 6dp; 2 more in `domain/economics/reserve_army/accumulation.py:115,121`, real `mechanization_displacement`/`firm_failures` rounding); 1 presentational (`domain/economics/tick/precarity.py:32`, `>>> round(u6, 2)` doctest, not real) | `single_county` | fully intrinsic, no BSL layer | |
| 6 | ReserveArmySystem | HYBRID | 147 | **sigmoids** (spec §6.4) | none found | none found at 1 hop (`domain.economics.reserve_army.calculator` has no `round()` — the 2 real rounds under `reserve_army/accumulation.py` are consumed via TickDynamicsSystem's pipeline, not this system's own import) | law:tests/unit/engine/laws/test_law_reserve_army.py (no territory carries positive `reserve_ratio`; `RESERVE_ARMY_PRESSURE` never fires; Task 11 property laws cover zero-ratio inactivity, the wage_pressure clamp/positive-wage floor, sigmoid monotonicity in reserve_ratio, and event/node payload agreement) | BSL guards/thresholds + named intrinsic core (sigmoid) | |
| 7 | CommunitySystem | HYBRID | 675 | none named in spec §6.4 inventory | none found | **0 in-tick**; 4 presentational (`formulas/community.py:68,101,103,133` — all `>>> round(...)` doctests, not real production rounding) | law:tests/unit/engine/laws/test_law_community_system.py (`community_hypergraph` is `None` by default; no MEMBERSHIP edges seeded; `step()` returns on its first/second guard clause every tick; Task 11 property laws cover both inactivity gates (no config, no memberships), threat_score bounds, and decay bounds/non-increase) | BSL guards/thresholds + named intrinsic core (`formulas.consciousness`, `formulas.community`) | |
| 8 | LifecycleSystem | BSL_RULES | 293 | none named in spec §6.4 inventory | none found | none found at 1 hop (imports `domain.economics.lifecycle.{cohort_dynamics,inheritance,legitimation,mobility,types}` — none of these carry `round()`; the 2 hits under `formulas/lifecycle.py` are a *different*, unimported module — doctests only, confirmed by read) | `imperial_circuit`, `two_node` | pure BSL rules, no named intrinsic | |
| 9 | SolidaritySystem | BSL_RULES | 202 | none named in spec §6.4 inventory | none found | none found | law:tests/unit/engine/laws/test_law_solidarity.py (every SOLIDARITY edge across all five scenarios has `solidarity_strength=0.0`; the transmission loop's early-continue guard skips every edge every tick; Task 11 property laws cover the [0,1] clamp, directional convergence without overshoot, activation-threshold inactivity, and inertia on dead nodes/zero infrastructure) | pure BSL rules, no named intrinsic | |
| 10 | ImperialRentSystem | HYBRID | 836 (`engine/systems/economic.py`) | none named in spec §6.4 inventory | none found directly; gates an **optional** sub-stage (`_invoke_n_if_wired` → `engine/systems/n.py` / `Vol2CirculationStep`, ADR-tracked as `n_vol2_step` in `sentinels/seam_algebra/registry.py`) whose own transitive imports (incl. `domain/economics/circulation/turnover.py`, 5 `round()` hits) were **not traced past this point** — flagged for the Task 17 deep audit, not resolved here | none found in `economic.py` itself; the gated `n.py` sub-stage's `round()` sites are unresolved (flagged above) | `fascist_bifurcation`, `glut`, `imperial_circuit`, `starvation`, `two_node` | BSL guards/thresholds + named intrinsic core (rent-pool tensor math) | |
| 11 | TransportSystem | HYBRID | 185 | none named in spec §6.4 inventory | none found | none found | law:tests/unit/engine/laws/test_law_transport.py (P26 U5e default-OFF, `TransportDefines.enabled=False`; no canonical scenario enables the corridor mesh — byte-identical by design; Task 11 property laws now cover decay-clamp, demand-signal non-negativity, inactivity, and connectivity-aggregation invariants) | BSL guards/thresholds + named intrinsic core (`domain.geography.corridor_mesh`) | |
| 12 | DispossessionEventSystem | BSL_RULES | 141 | none named in spec §6.4 inventory | none found | none found | law:tests/unit/engine/laws/test_law_dispossession_events.py (no territory carries positive `foreclosure_rate`/`eviction_rate`/`displacement_rate`; `DISPOSSESSION_EVENT`/`VALUE_TRANSFER` never fire; Task 11 property laws cover value-transfer conservation, the wealth clamp/non-negativity, zero-activity-rate inactivity, and intensity bounds+monotonicity) | pure BSL rules, no named intrinsic | |
| 13 | DecompositionSystem | BSL_RULES | 369 | none named in spec §6.4 inventory | none found | none found | law:tests/unit/engine/laws/test_law_decomposition_system.py (`SUPERWAGE_CRISIS`/`CLASS_DECOMPOSITION` never fire in 150 ticks on any of the five scenarios; Task 11 property laws cover the bounded population split under default carceral fractions, monotone accumulation on the enforcer/internal-proletariat targets, one-time idempotence, and inactivity on an absent/exhausted LA source) | pure BSL rules, no named intrinsic | |
| 14 | ControlRatioSystem | BSL_RULES | 247 | none named in spec §6.4 inventory | none found | none found | law:tests/unit/engine/laws/test_law_control_ratio.py (gated entirely behind `persistent_data['_class_decomposition_tick']`, set only by a successful DecompositionSystem run, which never happens in these five scenarios; resolves automatically once that gap closes; Task 11 property laws cover the crisis threshold clamp, empty-prisoner inactivity, avg_organization monotonicity, and the crisis/terminal-decision idempotent latch) | pure BSL rules, no named intrinsic | |
| 15 | MetabolismSystem | BSL_RULES | 153 | none named in spec §6.4 inventory | none found | none found | law:tests/unit/engine/laws/test_law_metabolism.py (writes `biocapacity`/`max_biocapacity` onto TERRITORY nodes only, unobservable via `entity_delta`; `ECOLOGICAL_OVERSHOOT` never fires within the tested horizon on any of the five scenarios; Task 11 property laws cover the biocapacity clamp, the hysteresis ratchet's non-increase, zero-forcing inactivity, and inactive-social_class exclusion from consumption) | pure BSL rules, no named intrinsic | |
| 16 | OODASystem | HYBRID | 478 | **seeded RNG draws** (spec §6.4) | **yes** — transitively: `ooda/npc_stub.py::select_npc_actions` threads `rng_seed` into `ooda/state_ai/decision.py:513` (`random.Random(rng_seed)`, plus a tiebreaker `rng.uniform(0.0, 0.01)` at :553), `ooda/state_ai/repress_effects.py:107,200,276`, `ooda/state_ai/administer_effects.py:138` (all `random.Random(rng_seed)` / `rng.random()`) | none found in `ooda.py` itself; `ooda/state_ai/observability.py` and `ooda/state_ai/faction_dynamics.py` carry `round()` hits but are not imported by `ooda.py` directly (faction_dynamics is pulled in via ElectoralSystem, row 24, not here) — none found at 1 hop from `ooda.py` | law:tests/unit/engine/laws/test_law_ooda.py (no ORGANIZATION nodes seeded in any canonical scenario; `ORGANIZATIONAL_ACTION` fires every tick with `org_count=action_count=layer0_count=0` — control flow runs, no organizational logic exercises; Task 11 property laws cover initiative-score conservation, Business-org action-phase exclusion, the 500-action clamp, and org-free inactivity) | BSL guards/thresholds + named intrinsic core (initiative scoring, RNG-seeded action selection) | |
| 17 | FactionInfluenceSystem | HYBRID | 256 | **seeded RNG draws** (spec §6.4) | **yes** — direct `resolve_rng(services, tick)` call at `faction_influence.py:68` | none found in-tick (`formulas/balkanization.py:386` is a `>>> round(...)` doctest, not real; confirmed by read) | law:tests/unit/engine/laws/test_law_faction_influence.py (no FACTION nodes seeded; `winning_faction_for_territory` returns `None` for every territory every tick; no `TERRITORY_TRANSITION`/`FACTION_VICTORY`/`RED_SETTLER_TRAP_DETECTED`/`SECESSION_DECLARED` fires; Task 11 property laws cover argmax soundness, unchanged-winner inactivity, FACTION_VICTORY threshold soundness+uniqueness, and empty-graph inactivity) | BSL guards/thresholds + named intrinsic core (balkanization lattice) | |
| 18 | DoctrineSystem | BSL_RULES | 700 | none named in spec §6.4 inventory | **yes** — direct `resolve_rng(services, tick)` call at `doctrine.py:673` | none found | law:tests/unit/engine/laws/test_law_doctrine_system.py (no ORGANIZATION nodes seeded; module docstring documents this as a no-op on the qa:regression goldens for exactly this reason; Task 11 property laws cover org-less inactivity, the theoretical_labor non-negativity clamp, acquired-set monotonicity on an ordinary tick, and graph-shape invariance) | pure BSL rules, no named intrinsic (RNG use is a rules-shaped weighted draw, not a numeric core) | |
| 19 | SurvivalSystem | HYBRID | 165 | **sigmoids** (spec §6.4) | none found | none found | `imperial_circuit`, `starvation`, `two_node` | BSL guards/thresholds + named intrinsic core (survival-calculus sigmoid) | |
| 20 | StruggleSystem | BSL_RULES | 748 | **seeded RNG draws** (spec §6.4) | **yes** — direct `resolve_rng(services, tick)` call at `struggle.py:299` | none found (imports `formulas.reactionary`, `domain.bifurcation.legitimation` — neither carries a `round()` hit) | `fascist_bifurcation`, `glut`, `imperial_circuit`, `two_node` | pure BSL rules, no named intrinsic (RNG use is a rules-shaped weighted draw) | |
| 21 | ConsciousnessSystem | HYBRID | 233 (`domain/bifurcation/consciousness.py`) | **Shannon entropy** (spec §6.4) | none found | none found | `fascist_bifurcation`, `glut`, `imperial_circuit`, `two_node` | BSL guards/thresholds + named intrinsic core (entropy calculation) | |
| 22 | FascistFactionSystem | BSL_RULES | 354 (`engine/systems/reactionary.py`) | **seeded RNG draws** (spec §6.4) | **yes** — direct `resolve_rng(services, tick)` call at `reactionary.py:241` | none found (`formulas/balkanization.py:386` doctest only, shared with row 17) | `fascist_bifurcation`, `imperial_circuit` | pure BSL rules, no named intrinsic (RNG use is a rules-shaped weighted draw) | |
| 23 | AllegianceSystem | BSL_RULES | 516 | **sqrt** (spec §6.4) | none found | none found (`formulas.politics` carries no `round()` hit) | `bernie_valve`, `debs`, `mitterrand`, `syriza`, `weimar` | pure BSL rules, no named intrinsic (sqrt is a bounded formula call, not a structural numeric core) | |
| 24 | ElectoralSystem | HYBRID | 1,269 | **seeded RNG draws** (spec §6.4) | **yes** — direct `resolve_rng(services, tick)` typed as `random.Random` at `electoral.py:841` | **10 in-tick, all real (no doctests)**: 4 in `domain/institution/balance.py:86-89` (`update_internal_balance`, rounding 4 faction-balance fractions to 6dp) + 6 in `ooda/state_ai/faction_dynamics.py:237-239,510-512` (`renormalize_faction_balance`, imported locally inside `electoral.py:1029`, rounding to 6dp) | `bernie_valve`, `debs`, `mitterrand`, `syriza`, `weimar` | BSL guards/thresholds + named intrinsic core (turnout/competitiveness formulas, faction-balance renormalization) | |
| 25 | PolicySystem | BSL_RULES | 782 | none named in spec §6.4 inventory | none found | none found | `bernie_valve`, `mitterrand`, `syriza` | pure BSL rules, no named intrinsic | |
| 26 | SovereigntySystem | BSL_RULES | 160 | none named in spec §6.4 inventory | none found | none found in-tick (`formulas/balkanization.py:386` doctest only, shared with rows 17/22) | law:tests/unit/engine/laws/test_law_sovereignty.py (no SOVEREIGN nodes seeded; CLAIMS-based effective-controller/metabolic-impact resolution and `DUAL_POWER_ACTIVE` never exercise; the unconditional empty-dict `persistent_data` write every tick is bookkeeping, not material logic; Task 11 property laws cover claims-gated inactivity, effective-controller max/tiebreak resolution, no-double-counting of metabolic impact, and dual-power emission count) | pure BSL rules, no named intrinsic | |
| 27 | MarketScissorsSystem | RUST_INTRINSIC | 582 | **tanh + log** (spec §6.4) | none found | none found at 1 hop (`formulas.market` carries no `round()` hit) | `imperial_circuit`, `single_county`, `two_node` | fully intrinsic, no BSL layer | |
| 28 | ContradictionSystem | HYBRID | 1,127 | none named in spec §6.4 inventory | none found | **0 in-tick**; 1 presentational (`formulas/contradiction.py:53`, `>>> round(...)` doctest, not real) | `detroit_tri_county` | BSL guards/thresholds + named intrinsic core (dialectics core coupling/opposition/regime, working-day resolver) | |
| 29 | ContradictionFieldSystem | RUST_INTRINSIC | 259 | none named in spec §6.4 inventory (field-derivative family; see row 30) | none found | none found | `imperial_circuit`, `two_node` | fully intrinsic, no BSL layer | |
| 30 | FieldDerivativeSystem | RUST_INTRINSIC | 457 | none named in spec §6.4 inventory (field-derivative family; see row 29) | none found | none found | `imperial_circuit` | fully intrinsic, no BSL layer | |
| 31 | CollapseTransitionSystem | BSL_RULES | 313 | none named in spec §6.4 inventory | none found | none found in-tick (`formulas/balkanization.py:386` doctest only, shared with rows 17/22/26) | law:tests/unit/engine/laws/test_law_collapse_transition.py (no SOVEREIGN nodes seeded; `SOVEREIGN_COLLAPSE`/`CIVIL_WAR_DECLARED`/`TERRITORY_TRANSITION` never fire; Task 11 property laws cover Territory-node conservation, exact/exclusive post-collapse CLAIMS partition, old-Sovereign claims stripping, and inactivity on a healthy graph) | pure BSL rules, no named intrinsic | |
| 32 | EdgeTransitionSystem | BSL_RULES | 894 (`engine/systems/edge_transition/`: 36 `__init__.py` + 858 `_legacy.py`) | none named in spec §6.4 inventory | none found | none found | law:tests/unit/engine/laws/test_law_edge_transition.py (no edge in any of the five scenarios carries an `edge_mode` attribute — `Relationship` does not declare that field; the 17-transition predicate table never evaluates a real transition; Task 11 property laws cover state-machine transition-pair closure, max-priority selection among simultaneously-eligible transitions, CO-OPTIVE suppression-accumulator monotonicity, and latent-release amplification) | pure BSL rules, no named intrinsic | |
| 33 | WealthDistributionSystem | RUST_INTRINSIC | 274 | none named in spec §6.4 inventory | none found | **0 in-tick**; 1 presentational (`formulas/class_dynamics.py:225`, `>>> round(...)` doctest, not real) | `imperial_circuit`, `two_node` | fully intrinsic, no BSL layer | |
| 34 | EpistemicHorizonSystem | BSL_RULES | 245 | none named in spec §6.4 inventory | none found | none found | law:tests/unit/engine/laws/test_law_epistemic_horizon.py (writes shadow `mass_receptivity`/`intel_confidence`/`vision_state` onto TERRITORY nodes only, not `state.entities`; emits no events; Phase-1 observe-only shadow per its own docstring; Task 11 property laws cover honest inactivity, mass_receptivity/intel_confidence bounds, and the vision_state threshold table) | pure BSL rules, no named intrinsic | |

## Corrections against the injected Task-2 tick-profile evidence

Task 2's tick-profile handoff describes 23/34 systems as carrying a ratified
`budget.json` ceiling and lists the remainder (`AllegianceSystem`,
`DoctrineSystem`, `ElectoralSystem`, `PolicySystem`, `MarketScissorsSystem`,
`WealthDistributionSystem`, `EpistemicHorizonSystem`, `CommunitySystem`) as
uncovered by that budget file — a **separate** gap axis (perf-budget
coverage) from this table's coverage-instrument column (scenario/property-law
coverage). The two axes are not the same: e.g. `MarketScissorsSystem` has a
ratified perf budget missing from `budget.json`'s named list per Task 2, but
**does** have scenario coverage per this table (`imperial_circuit`,
`single_county`, `two_node`); `CommunitySystem` is gapped on **both** axes.
This table intentionally does not merge the two — Task 2's report
(`reports/tick-profile-2026-07-29.md`, if landed) is the perf-budget axis's
home.

## Corrections against the injected Task-6 handoff

Task 6 (TickContext key census) is marked `UNAVAILABLE` in this run's inputs
(the upstream agent failed). No TickContext-key column was requested for this
table (spec §8.4's named columns are classification, LOC, float hazards, RNG
usage, `round()` sites, coverage instrument, BSL-vs-intrinsic split,
sign-off), so this gap does not block Task 10 — it blocks whichever task
depends on the key census directly.

## Open items for Task 17 (pre-freeze deep audit)

1. `ImperialRentSystem`'s gated `n.py` (`Vol2CirculationStep`) sub-stage's
   `round()`/RNG closure was not traced past one hop in this scaffold — resolve
   before the freeze tag.
2. `OODASystem`'s `ooda/state_ai/observability.py` (3 `round()` hits) was not
   confirmed reachable or unreachable from the tick path — resolve before the
   freeze tag.
3. All 17 `CoverageGap` rows above are Task 11's floor — the freeze tag blocks
   until each carries a property law, a transcribed unit suite, or a
   Director-signed waiver row (fixes M14, spec §8).
