---
description: "Task list for spec-070 Sovereign Topology + Faction Influence + Balkanization"
---

# Tasks: Sovereign Topology + Faction Influence + Balkanization

**Input**: Design documents from `/specs/070-balkanization/`
**Prerequisites**: [plan.md](./plan.md) ✓, [spec.md](./spec.md) ✓, [research.md](./research.md) ✓, [data-model.md](./data-model.md) ✓, [contracts/](./contracts/) ✓, [quickstart.md](./quickstart.md) ✓

**Tests**: Project follows TDD per `babylon/CLAUDE.md` (Red → Green → Refactor mandatory). Test tasks are FIRST in each phase; tests MUST fail before implementation begins.

**Organization**: Tasks are grouped by user story (US1–US4 from spec.md) plus Setup, Foundational, and Polish phases. Each story is independently testable per its "Independent Test" in spec.md.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Different file, no dependency on incomplete tasks → safe to parallelize.
- **[Story]**: User story label (US1, US2, US3, US4). Setup / Foundational / Polish have no story label.

## Path Conventions

Single project — Babylon engine library. All paths relative to repo root `/home/user/projects/game/babylon/`.
- Source: `src/babylon/...`
- Tests: `tests/unit/balkanization/`, `tests/integration/balkanization/`, `tests/scenario/balkanization/`
- Seed data: `src/babylon/data/game/balkanization/`
- Migrations: `src/babylon/persistence/migrations/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the subsystem directory layout. No new deps; existing project bootstrap suffices (poetry install, mise install — already done).

- [ ] T001 Create new module directories: `src/babylon/data/game/balkanization/`, `tests/unit/balkanization/`, `tests/integration/balkanization/`, `tests/scenario/balkanization/`
- [ ] T002 [P] Add `__init__.py` stub files to each new directory (empty for now — populated by foundational tasks)
- [ ] T003 [P] Verify existing pytest markers in `pyproject.toml` are sufficient (math, ledger, topology, integration, unit, scenario) — no new marker required

**Checkpoint**: Directory skeleton exists.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core enums, entities, defines, migration, and GraphProtocol extensions that EVERY user story depends on.

**⚠️ CRITICAL**: No user story work begins until this phase is complete.

### Enums (parallel-safe — different files)

- [ ] T004 [P] Create `src/babylon/models/enums/balkanization.py` with `ColonialStance`, `ExtractionPolicy`, `SovereigntyType`, `FiscalStatus`, `LegalStatus`, `SupportType`, `PlayerMode` StrEnums per data-model.md §1.1-1.7
- [ ] T005 [P] Test enum value sets in `tests/unit/balkanization/test_enums.py` — verify each enum has exactly the documented members and no extras (per FR-002, FR-003, FR-006, FR-010, FR-011, FR-015)
- [ ] T006 Extend `src/babylon/models/enums/topology.py` `EdgeType` with `CLAIMS = "claims"`, `INFLUENCES = "influences"`, `ADMINISTERS = "administers"` per data-model.md §1.8
- [ ] T007 Extend `src/babylon/models/enums/events.py` `GameOutcome` with `RED_OGV = "red_ogv"`, `FRAGMENTED_COLLAPSE = "fragmented_collapse"` per data-model.md §1.9 (PRESERVE existing 4 values)
- [ ] T008 Extend `src/babylon/models/enums/events.py` `EventType` with 9 new values: `SOVEREIGN_COLLAPSE`, `TERRITORY_TRANSITION`, `FACTION_VICTORY`, `SECESSION_DECLARED`, `CIVIL_WAR_DECLARED`, `RED_SETTLER_TRAP_DETECTED`, `DUAL_POWER_ACTIVE`, `RED_OGV_ENDGAME`, `FRAGMENTED_COLLAPSE_ENDGAME` per data-model.md §1.10
- [ ] T009 [P] Update `src/babylon/models/enums/__init__.py` re-exports to surface the new enums (`ColonialStance`, `ExtractionPolicy`, `SovereigntyType`, `FiscalStatus`, `LegalStatus`, `SupportType`, `PlayerMode`)

### Config / Defines

- [ ] T010 Write test `tests/unit/balkanization/test_balkanization_defines.py` for `BalkanizationDefines` model — verify all 17 fields exist with documented defaults from `contracts/balkanization_defines.schema.json` (RED phase)
- [ ] T011 Create `src/babylon/config/defines/balkanization.py` with `BalkanizationDefines` Pydantic frozen model per `contracts/balkanization_defines.schema.json` (17 fields with theoretical defaults from balkanization-spec.yaml v1.2.0, per research.md R-001)

### Formula module

- [ ] T012 Write test `tests/unit/balkanization/test_metabolic_impact_formula.py` for `calculate_metabolic_impact(extraction_policy)` — verify INTENSIFY → −0.02, CONTINUE → −0.005, CEASE → +0.01 per FR-004 (RED)
- [ ] T013 Create `src/babylon/formulas/balkanization.py` with `calculate_metabolic_impact(policy: ExtractionPolicy, defines: BalkanizationDefines) -> float` (GREEN)
- [ ] T014 [P] Update `src/babylon/formulas/__init__.py` `__all__` to export `calculate_metabolic_impact`

### Entities

- [ ] T015 [P] Write test `tests/unit/balkanization/test_faction_entity.py` — verify `PoliticalFaction` Pydantic model has all fields per data-model.md §2.1, validates id pattern `^FAC_[A-Z][A-Z0-9_]*$`, frozen=True, stance-multiplier consistency validator (RED)
- [ ] T016 [P] Write test `tests/unit/balkanization/test_sovereign_entity.py` — verify `Sovereign` Pydantic model has all fields per data-model.md §2.2, validates id pattern, computed `metabolic_impact` property, extraction-policy-vs-ruling-faction validator (RED)
- [ ] T017 Create `src/babylon/models/entities/political_faction.py` with `PoliticalFaction(BaseModel)` per data-model.md §2.1 (GREEN); name avoids `FactionBalance` / `StateFaction` collision per FR-045
- [ ] T018 Create `src/babylon/models/entities/sovereign.py` with `Sovereign(BaseModel)` per data-model.md §2.2 including `@computed_field` for `metabolic_impact` (GREEN)
- [ ] T019 Update `src/babylon/models/entities/__init__.py` to re-export `PoliticalFaction` and `Sovereign`

### GraphProtocol extensions

- [ ] T020 Write test `tests/unit/balkanization/test_graph_protocol_extensions.py` for each of the 6 new methods per `contracts/graph_protocol_extensions.md`: `query_faction_influence_by_territory`, `query_sovereign_claims`, `query_territory_claims`, `query_adjacent_territories`, `bulk_partition_claims`, `query_contiguous_component_under_predicate` — verify type signatures + determinism guarantees (RED)
- [ ] T021 Extend `src/babylon/engine/graph_protocol.py` `GraphProtocol` Protocol with the 6 new abstract methods; raise `NotImplementedError` default
- [ ] T022 Extend `src/babylon/engine/networkx_adapter.py` (or wherever `NetworkXAdapter` lives) with concrete implementations of the 6 methods, honoring deterministic-ordering guarantees per `graph_protocol_extensions.md` (GREEN)
- [ ] T023 [P] Write Hypothesis property test in `tests/unit/balkanization/test_fracture_operation_o1.py` verifying `bulk_partition_claims` cost is O(K) in moving territory count, NOT O(N) in unchanged-territory count — benchmark at N ∈ {10, 100, 1000} per SC-004 (this test will be activated in US4 but the bare scaffolding lands here for the protocol test)

### Persistence

- [ ] T024 Create migration `src/babylon/persistence/migrations/00XX_balkanization.sql` (next sequential number after spec-069's migrations) — adds: `runtime_political_factions` table, `runtime_sovereigns` table, `runtime_claims_edges` table, `runtime_influences_edges` table, `runtime_administers_edges` table, `balkanization_claims_audit` table, `balkanization_influences_audit` table per research.md R-005 DDL
- [ ] T025 Write integration test `tests/integration/balkanization/test_postgres_persistence.py` — verify migration applies cleanly + each table has expected schema + indexes (RED)
- [ ] T026 Create `src/babylon/persistence/balkanization_history.py` with `record_claims_mutation(tick, edge_state, op, observer=False)` and `record_influences_mutation(tick, edge_state, op, observer=False)` functions for post-tick audit-row writes per FR-046 (GREEN — uses existing psycopg pool from spec-037)
- [ ] T027 [P] Write integration test `tests/integration/balkanization/test_audit_round_trip.py` — write a CLAIMS mutation, read it back from `balkanization_claims_audit`, verify `observer_mutation` flag preserves correctly per FR-049

### Seed data + loaders

- [ ] T028 [P] Create `src/babylon/data/game/balkanization/seed_factions.json` with 4 canonical PoliticalFactions per data-model.md §8: FAC_RESTORATIONIST (UPHOLD), FAC_WORKERS_CONGRESS (IGNORE), FAC_DECOLONIAL (ABOLISH), FAC_LIBERAL_IMPERIAL (IGNORE) per research.md R-002. Validate against `contracts/seed_factions.schema.json`
- [ ] T029 [P] Create `src/babylon/data/game/balkanization/seed_sovereigns.json` with 3 starting Sovereigns per FR-040 / FR-040a / FR-040b: SOV_USA_FED (ruled by FAC_RESTORATIONIST, INTENSIFY, all-Detroit-tri-county DE_JURE claims at control_level=1.0), SOV_CAN_FED (ruled by FAC_LIBERAL_IMPERIAL, CONTINUE, claims `canada` boundary + cross-border claims), and SOV_EXTERIOR_NULL (PROVISIONAL, NULL ruling_faction, CONTINUE policy, claims `rest_of_usa` boundary at DE_JURE / control_level=1.0). Validate against `contracts/seed_sovereigns.schema.json` (minItems=3). The Sovereign Pydantic validator from T018 MUST be updated to permit `ruling_faction_id=NULL` paired with `extraction_policy=CONTINUE` (special-cased combination per FR-040b).
- [ ] T030 Write test `tests/unit/balkanization/test_seed_loaders.py` — verify `load_seed_factions()` and `load_seed_sovereigns()` return correctly-typed lists, validate JSON-Schema conformance (RED)
- [ ] T031 Create `src/babylon/data/game/balkanization/__init__.py` exporting `load_seed_factions() -> list[PoliticalFaction]` and `load_seed_sovereigns() -> list[Sovereign]` with JSON-Schema validation on load (GREEN)
- [ ] T032 Extend `src/babylon/persistence/postgres_initialization.py` to invoke `load_seed_factions()` + `load_seed_sovereigns()` and insert into the new runtime tables during db-init, alongside the existing `DOMESTIC_REST_NODE` initialization

### Event payload schemas (parallel-safe — different files)

- [ ] T033 [P] Write `src/babylon/models/events/balkanization_payloads.py` containing 9 Pydantic frozen `BaseModel` classes for the new event payloads per `contracts/balkanization_events.json` schemas (`SovereignCollapsePayload`, `TerritoryTransitionPayload`, etc.)
- [ ] T034 [P] Write test `tests/unit/balkanization/test_event_payloads.py` — validate each payload class against the JSON-Schema in `contracts/balkanization_events.json` (RED+GREEN combined)

### Helper formulas (parallel-safe — same file but logically grouped writes)

- [ ] T035 Write tests `tests/unit/balkanization/test_stance_mappings.py` for both derive-functions — exhaustive table verification of (ColonialStance → ExtractionPolicy) per FR-003 + data-model.md §3.2 and (ColonialStance → mechanical multipliers) per FR-007 + data-model.md §3.1 (RED — tests fail until T036+T037 land)
- [ ] T036 Extend `src/babylon/formulas/balkanization.py` with `derive_extraction_policy_from_stance(stance: ColonialStance) -> ExtractionPolicy` per FR-003 + data-model.md §3.2 (GREEN for the policy half of T035)
- [ ] T037 Extend `src/babylon/formulas/balkanization.py` with `derive_default_multipliers_from_stance(stance: ColonialStance, defines: BalkanizationDefines) -> tuple[float, float, float, float]` per FR-007 + data-model.md §3.1 (GREEN for the multipliers half of T035)

**Checkpoint**: All enums, entities, defines, GraphProtocol extensions, migration, audit writer, seed loaders, event payloads, and helper formulas in place. User-story implementation can now begin.

---

## Phase 3: User Story 1 — Extraction Policy Decides a Territory's Material Fate (Priority: P1) 🎯 MVP

**Goal**: A Sovereign's `ExtractionPolicy` applies a deterministic per-tick effect to claimed Territories' habitability, producing visibly different trajectories under INTENSIFY / CONTINUE / CEASE.

**Independent Test**: Seed one Sovereign with `ExtractionPolicy=INTENSIFY` claiming one Territory at habitability=0.8. Tick 10 times; habitability falls by ≈0.2. Switch to CONTINUE; slope flattens. Switch to CEASE; habitability recovers. No other system involvement required.

### Tests for User Story 1

- [ ] T038 [P] [US1] Write test `tests/unit/balkanization/test_sovereignty_system.py` for `SovereigntySystem.tick(graph, services, context)` — verify it reads each Sovereign's claims, computes `metabolic_impact`, writes `context.persistent_data["balkanization.metabolic_impact_by_territory"]` per system_io_contracts.md §2 (RED)
- [ ] T039 [P] [US1] Write test `tests/integration/balkanization/test_metabolism_extension.py` — verify MetabolismSystem applies the sovereign-driven additive term to `territory.habitability` per FR-019 + FR-043 (RED)
- [ ] T040 [P] [US1] Write integration test `tests/integration/balkanization/test_us1_extraction_trajectory.py` per spec.md US1 Acceptance Scenarios 1–4: INTENSIFY drops habitability 0.2 over 10 ticks; CEASE reverses slope within 5 ticks; multiple Territories all receive effect; unclaimed Territory unaffected (RED)
- [ ] T041 [US1] Write test `tests/unit/balkanization/test_dual_power_tiebreak.py` — verify FR-020: when multiple CLAIMS edges target same Territory, only the highest-`control_level` Sovereign's metabolic_impact applies (no double-counting). Tiebreaker by Sovereign ID lexicographic (RED)

### Implementation for User Story 1

- [ ] T042 [US1] Create `src/babylon/engine/systems/sovereignty.py` with `SovereigntySystem.tick()` implementing the read/write contract from system_io_contracts.md §2 (basic version: per-Sovereign metabolic_impact computation; CLAIMS-update + ruling_faction install logic added in later phases). GREEN for T038.
- [ ] T043 [US1] Extend `src/babylon/engine/systems/metabolism.py` per FR-043 + system_io_contracts.md §4: read `context.persistent_data["balkanization.metabolic_impact_by_territory"]` and apply additive term to `territory.habitability` BEFORE existing biocapacity-Δ computation. GREEN for T039.
- [ ] T044 [US1] Register `SovereigntySystem` in `src/babylon/engine/simulation_engine.py` `_DEFAULT_SYSTEMS` at position ~17.5 (between ConsciousnessSystem at 17 and ContradictionSystem at 18). Add `SovereigntySystem` to `CONSEQUENCE_SYSTEMS` frozenset per FR-042. Verify spec-056 import-time partition-integrity assertion still passes.
- [ ] T045 [US1] Implement effective-controller resolution in `SovereigntySystem` per FR-020 — populate `context.persistent_data["balkanization.effective_controller_by_territory"]` using `query_territory_claims` (highest control_level + lexicographic tiebreaker). GREEN for T041.
- [ ] T046 [US1] Emit `DUAL_POWER_ACTIVE` event per FR-035 from `SovereigntySystem` when ≥2 CLAIMS edges have `control_level > 0.0` on a Territory.
- [ ] T047 [US1] Verify US1 integration test (T040) passes end-to-end; if not, iterate on systems until the 4 acceptance scenarios all pass.

**Checkpoint**: User Story 1 fully functional. A Sovereign installed at game start drives per-tick habitability changes deterministically. SC-003 verifiable in isolation. MVP-shippable.

---

## Phase 4: User Story 2 — Factions Contest a Territory and Install Sovereigns (Priority: P2)

**Goal**: Multiple Factions accumulate INFLUENCES edges into the same Territory; the winning Faction installs its preferred Sovereign whose ExtractionPolicy then drives habitability per US1.

**Independent Test**: Seed three Factions with overlapping INFLUENCES on one Territory under a collapsing Sovereign. Tick. The highest-influence Faction installs a Sovereign with the corresponding ExtractionPolicy. Flipping influence flips the Sovereign + emits `TERRITORY_TRANSITION`.

### Tests for User Story 2

- [ ] T048 [P] [US2] Write test `tests/unit/balkanization/test_winning_faction_argmax.py` — verify `winning_faction_for_territory(graph, territory_id)` formula per FR-021: argmax of Σ INFLUENCES.influence_level, with incumbent-priority tiebreaker first, then seed-deterministic RNG (RED)
- [ ] T049 [P] [US2] Write test `tests/unit/balkanization/test_faction_influence_system.py` for `FactionInfluenceSystem.tick()` — verify it computes per-Territory winning-Faction snapshot, writes `context.persistent_data["balkanization.winning_faction_by_territory"]`, emits `TERRITORY_TRANSITION` on flip (RED)
- [ ] T050 [P] [US2] Write integration test `tests/integration/balkanization/test_us2_faction_competition.py` per spec.md US2 Acceptance Scenarios 1–5: influence-distribution flips → correct winning Faction → new Sovereign installed with derived ExtractionPolicy → `TERRITORY_TRANSITION` emitted → tied factions resolved by tiebreaker → initial control_level=0.8 + legal_status=DE_FACTO → sum-of-influences not capped at 1.0 (RED)
- [ ] T051 [P] [US2] Write test `tests/unit/balkanization/test_red_settler_trap_detector.py` — verify `RED_SETTLER_TRAP_DETECTED` fires for a Faction with `class_reduction ≥ 0.6 AND colonial_stance ∈ {UPHOLD, IGNORE}` per FR-034 + SC-006 (RED)

### Implementation for User Story 2

- [ ] T052 [US2] Extend `src/babylon/formulas/balkanization.py` with `winning_faction_for_territory(graph, territory_id, incumbent_faction_id, rng) -> str` per FR-021. GREEN for T048.
- [ ] T053 [US2] Extend `src/babylon/formulas/balkanization.py` with `detect_red_settler_trap(faction: PoliticalFaction, defines: BalkanizationDefines) -> bool` per FR-034. GREEN for T051.
- [ ] T054 [US2] Create `src/babylon/engine/systems/faction_influence.py` with `FactionInfluenceSystem.tick()` implementing system_io_contracts.md §1: read INFLUENCES edges, compute per-Territory winning-Faction snapshot, write to context.persistent_data, emit TERRITORY_TRANSITION on flip, emit RED_SETTLER_TRAP_DETECTED. GREEN for T049.
- [ ] T055 [US2] Extend `FactionInfluenceSystem` with FACTION_VICTORY event emission per FR-026: when a Faction's aggregate influence share crosses `faction_victory_supermajority_threshold` (BalkanizationDefines default 0.66).
- [ ] T056 [US2] Register `FactionInfluenceSystem` in `src/babylon/engine/simulation_engine.py` `_DEFAULT_SYSTEMS` at position ~14.5 (between OODASystem at 14 and SurvivalSystem at 15). Add to `CONSEQUENCE_SYSTEMS` frozenset per FR-042 + research.md R-003. Verify import-time partition assertion still passes.
- [ ] T057 [US2] Extend `SovereigntySystem.tick()` (from T042) to consume `context.persistent_data["balkanization.winning_faction_by_territory"]` and update `Sovereign.ruling_faction_id` + recompute `extraction_policy` on winning-Faction change. Emit appropriate audit rows via `balkanization_history.py`.
- [ ] T058 [US2] Verify US2 integration test (T050) passes end-to-end; iterate until all 5 acceptance scenarios pass.

**Checkpoint**: User Story 2 fully functional. Factions compete; winning Faction installs Sovereign; ExtractionPolicy derives from ColonialStance; transitions emit events. US1 + US2 work together: faction competition drives the metabolic_impact via the installed Sovereign.

---

## Phase 5: User Story 3 — Sovereign Collapse and Branching Endgames (Priority: P3)

**Goal**: When a Sovereign's legitimacy or habitability falls below thresholds, `SOVEREIGN_COLLAPSE` fires; contested territories transition; the global state drives the game to one of five distinct endgame outcomes.

**Independent Test**: Construct four synthetic terminal states (UPHOLD-majority + violence → FASCIST_CONSOLIDATION; IGNORE-majority + class-tension-down + habitability-crashing → RED_OGV; ABOLISH-majority + extraction-stopped + habitability-recovering → REVOLUTIONARY_VICTORY ["TRUE LIBERATION" framing]; no-majority + ≥3 sovereigns → FRAGMENTED_COLLAPSE). EndgameDetector emits the correct outcome from each; only one endgame per run.

### Tests for User Story 3

- [ ] T059 [P] [US3] Write test `tests/unit/balkanization/test_collapse_transition_system.py` for `CollapseTransitionSystem.tick()` per system_io_contracts.md §3: collapse predicate evaluation, 5-step partition pipeline per FR-024, deterministic ordering by Sovereign ID (RED)
- [ ] T060 [P] [US3] Write test `tests/integration/balkanization/test_collapse_transition_pipeline.py` per spec.md US3 Acceptance Scenarios 1–2: `legitimacy <= 0.0` triggers SOVEREIGN_COLLAPSE; ECOLOGICAL_OVERSHOOT event marks all overlap-zone Sovereigns for collapse in deterministic order; TERRITORY_TRANSITION fires per claimed Territory (RED)
- [ ] T061 [P] [US3] Write test `tests/unit/balkanization/test_endgame_revolutionary_victory_augmented.py` per FR-031: existing percolation + consciousness predicate plus new ABOLISH-Sovereign-majority + extraction-stopped + habitability-slope gate. Verify run that satisfies ONLY existing conditions but FAILS colonial-stance gate routes to RED_OGV instead (RED)
- [ ] T062 [P] [US3] Write test `tests/unit/balkanization/test_endgame_fascist_consolidation_augmented.py` per FR-031: existing false-consciousness route still fires + new political-violence route (UPHOLD-majority + state_violence_index=max + INTENSIFY policy) ALSO fires the same outcome (RED)
- [ ] T063 [P] [US3] Write test `tests/unit/balkanization/test_endgame_red_ogv.py` per FR-032: predicate requires IGNORE-Sovereign majority AND class_tension < floor AND habitability < floor AND habitability_slope < 0 (RED)
- [ ] T064 [P] [US3] Write test `tests/unit/balkanization/test_endgame_fragmented_collapse.py` per FR-032a: no-faction-majority AND active_sovereign_count ≥ 3 AND ≥1 sovereign of type {INSURGENT, OCCUPATION, EMERGENCY} AND duration ≥ 10 ticks (RED)
- [ ] T065 [P] [US3] Write test `tests/unit/balkanization/test_endgame_priority_order.py` per FR-033: when multiple predicates hold same tick, priority order is RED_OGV → FRAGMENTED_COLLAPSE → ECOLOGICAL_COLLAPSE → FASCIST_CONSOLIDATION → REVOLUTIONARY_VICTORY (RED)
- [ ] T066 [P] [US3] Write integration test `tests/integration/balkanization/test_endgame_predicates.py` per spec.md US3 Acceptance Scenarios 3–7: each of the 4 outcomes reachable from synthetic state; exactly one endgame event per run (RED)

### Implementation for User Story 3

- [ ] T067 [US3] Create `src/babylon/engine/systems/collapse_transition.py` with `CollapseTransitionSystem.tick()` per system_io_contracts.md §3: subscribe to ECOLOGICAL_OVERSHOOT + NUCLEAR_EXCHANGE; evaluate `legitimacy <= 0.0` predicate; emit SOVEREIGN_COLLAPSE; execute 5-step partition; emit TERRITORY_TRANSITION per claimed Territory; delete orphaned Sovereign nodes. GREEN for T059, T060.
- [ ] T068 [US3] Register `CollapseTransitionSystem` in `_DEFAULT_SYSTEMS` at position ~19.5 (between FieldDerivativeSystem at 20 and EdgeTransitionSystem at 21). Add to `CONSEQUENCE_SYSTEMS` frozenset per FR-042. Verify spec-056 assertion still passes.
- [ ] T069 [US3] Extend `src/babylon/engine/observers/endgame_detector.py`: AUGMENT existing `REVOLUTIONARY_VICTORY` predicate with the ABOLISH-Sovereign-majority + extraction_policy=CEASE + habitability_slope ≥ 0 gate per FR-031. Preserve existing percolation/class_consciousness conditions. GREEN for T061.
- [ ] T070 [US3] Extend `EndgameDetector`: AUGMENT existing `FASCIST_CONSOLIDATION` predicate with the second political-violence route (UPHOLD-Sovereign majority + state_violence_index=max + INTENSIFY policy) per FR-031. Existing false-consciousness route still fires the same outcome. GREEN for T062.
- [ ] T071 [US3] Extend `EndgameDetector` with new `RED_OGV` predicate per FR-032 (all 4 sub-conditions). GREEN for T063.
- [ ] T072 [US3] Extend `EndgameDetector` with new `FRAGMENTED_COLLAPSE` predicate per FR-032a (all 4 sub-conditions). GREEN for T064.
- [ ] T073 [US3] Implement priority-order logic in `EndgameDetector` per FR-033: evaluate predicates in order RED_OGV → FRAGMENTED_COLLAPSE → ECOLOGICAL_COLLAPSE → FASCIST_CONSOLIDATION → REVOLUTIONARY_VICTORY; first-match-wins; no second endgame after first fires. GREEN for T065.
- [ ] T074 [US3] Wire `RED_OGV_ENDGAME` and `FRAGMENTED_COLLAPSE_ENDGAME` events to fire from `EndgameDetector` with their full payloads per `contracts/balkanization_events.json`. For `REVOLUTIONARY_VICTORY` augmented path, include `user_facing_message: "TRUE LIBERATION achieved..."` in payload metadata per SC-012.
- [ ] T075 [US3] Verify US3 integration test (T066) passes end-to-end; iterate until all 7 acceptance scenarios pass.

### Scenario tests for User Story 3

- [ ] T076 [P] [US3] Write scenario test `tests/scenario/balkanization/test_five_endgames_reachable.py` — 100-run stochastic ensemble per SC-001 + SC-002. Given the FAC_RESTORATIONIST hard-start, FASCIST_CONSOLIDATION should be modal; verify each other endgame reachable at least once under stochastic player-equivalent input variation.
- [ ] T077 [P] [US3] Write scenario test `tests/scenario/balkanization/test_red_ogv_pedagogy.py` per SC-012 — scripted Workers' Congress path that satisfies RED_OGV predicate; verify the user-facing message names the contradiction between political victory and ecological defeat.

**Checkpoint**: User Story 3 fully functional. All 5 endgame outcomes distinguishable + reachable. Priority order enforced. User-facing TRUE_LIBERATION / FASCIST_VICTORY framings surface via payload metadata. SC-001, SC-002, SC-010, SC-012 all verified.

---

## Phase 6: User Story 4 — Secession and Civil War as O(1) Edge Rewiring (Priority: P4)

**Goal**: A Faction with strong but non-majority influence in a contiguous geographic sub-region of a Sovereign's territories can secede, fracturing one Sovereign into two via O(1) edge rewiring (per ADR029 + spec-070 R-004 H3 res-7 contiguity).

**Independent Test**: Construct a Sovereign claiming N=1000 Territories with a secessionist Faction holding influence_level > 0.5 in K=300 contiguous H3 hexes. Trigger fracture. Verify (a) two Sovereigns cover the full original set; (b) seceded Faction's territories migrate; (c) runtime scales with boundary size only, not with N (per SC-004 benchmark at N ∈ {10, 100, 1000}).

### Tests for User Story 4

- [ ] T078 [P] [US4] Write test `tests/unit/balkanization/test_contiguity_check.py` for `contiguous_influence_majority_subregion(graph, faction_id, sovereign_id, threshold, min_size)` formula per FR-029b: BFS over H3 res-7 hexes via existing `infrastructure/h3_mesh.py` `grid_disk` helpers; predicate `influence_level > threshold`; skip if region < min_contiguous_hex_count (RED)
- [ ] T079 [P] [US4] Activate Hypothesis property test `tests/unit/balkanization/test_fracture_operation_o1.py` (scaffolded in T023) — verify fracture cost flat in unchanged-territory count across N ∈ {10, 100, 1000} per FR-018 + SC-004 (RED)
- [ ] T080 [P] [US4] Write test `tests/unit/balkanization/test_hysteresis_buffer.py` — verify FR-029c: predicate must hold for `secession_hysteresis_ticks` consecutive ticks (default 3) before SECESSION_DECLARED fires; counter resets on predicate falsification (RED)
- [ ] T081 [P] [US4] Write integration test `tests/integration/balkanization/test_us4_secession_fracture.py` per spec.md US4 Acceptance Scenarios 1–5: 1000-territory parent + 300-contiguous secessionist → 2 sovereigns post-fracture with documented control_levels and legal_status; runtime flat in N; contested boundaries emit CIVIL_WAR_DECLARED + multiple DISPUTED CLAIMS; conquest via control_level reduction flips effective controller; orphaned Sovereign deleted in same tick (RED)

### Implementation for User Story 4

- [ ] T082 [US4] Extend `src/babylon/formulas/balkanization.py` with `contiguous_influence_majority_subregion(graph, faction_id, sovereign_id, threshold, min_size, defines) -> frozenset[str]` per FR-029b — BFS using `query_adjacent_territories` + `query_faction_influence_by_territory` helpers, deterministic ordering by hex ID lex per system_io_contracts.md §1 Determinism Notes. GREEN for T078.
- [ ] T083 [US4] Extend `FactionInfluenceSystem.tick()` (from T054) to evaluate active-secession predicate per FR-029a (2): for each (non-incumbent-Faction, Sovereign) pair, compute contiguous influence-majority sub-region; maintain hysteresis buffer per FR-029c; fire SECESSION_DECLARED when window elapses. GREEN for T080.
- [ ] T084 [US4] Extend `CollapseTransitionSystem.tick()` (from T067) with the active-secession branch: when consuming `context.persistent_data["balkanization.secession_eligible"]`, emit CIVIL_WAR_DECLARED and execute the fracture operation. GREEN for the relevant parts of T081.
- [ ] T085 [US4] Implement the fracture operation in `CollapseTransitionSystem` using `bulk_partition_claims` (T022): partition CLAIMS between parent and new Sovereign per FR-028; assign DE_FACTO + control_level=0.9 to seceded side, DISPUTED + control_level=0.1 to parent's residual claims on contested boundary; for fully-claimed contested Territories, leave both CLAIMS edges and rely on DUAL_POWER_ACTIVE diagnostic. GREEN for T079 + T081.
- [ ] T086 [US4] Implement the "secessionist holds 100% of parent's territories" edge case per spec.md: no second Sovereign produced; parent's `ruling_faction_id` is replaced by secessionist's preferred Faction; `extraction_policy` recomputed.
- [ ] T087 [US4] Implement orphaned-Sovereign cleanup in `CollapseTransitionSystem`: after any partition, if a Sovereign has zero CLAIMS, delete the node + outbound ADMINISTERS edges in the same tick per spec.md edge case + US4 AS5.
- [ ] T088 [US4] Implement conquest path: when an external Sovereign raises its CLAIMS.control_level above an incumbent's, `SovereigntySystem` emits TERRITORY_TRANSITION per US4 AS4. Verify integration with the dual-power tiebreak from T045.
- [ ] T089 [US4] Verify US4 integration test (T081) passes end-to-end; iterate until all 5 acceptance scenarios pass.

### Scenario tests for User Story 4

- [ ] T090 [P] [US4] Write scenario test `tests/scenario/balkanization/test_fracture_benchmark.py` per SC-004 — Hypothesis-driven benchmark at N ∈ {10, 100, 1000} claimed territories with same boundary size; assert runtime cost is bounded by K (boundary size), not N (total).

**Checkpoint**: User Story 4 fully functional. Active secession + collapse-driven fracture + conquest all work. O(1)-in-unchanged-territory-count verified empirically. SC-004 satisfied.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Player-mode plumbing, performance gates, determinism gates, docstrings, ai-docs sync, quickstart validation.

### Player modes (FR-047 — FR-050)

- [ ] T091 Write test `tests/unit/balkanization/test_player_mode_persistence.py` — verify CAMPAIGN + OBSERVER modes persist in run state, downstream systems can query mode, OBSERVER mutations bypass CAMPAIGN-mode resource-cost checks but are flagged in audit log (RED)
- [ ] T092 Implement `PlayerMode` persistence in run initial-state per FR-047 — extend the run-config schema with `player_mode: PlayerMode` and `player_faction_id: str | None` (None when mode=OBSERVER)
- [ ] T093 Wire CAMPAIGN-mode action routing through spec-072 stub: actions on Faction influence in CAMPAIGN mode raise `NotImplementedError("spec-072 integration pending")` with explicit pointer; OBSERVER-mode actions go directly to graph mutations via the protocol per FR-049
- [ ] T094 Implement audit-flag for OBSERVER mutations per FR-049: every `balkanization_history.py` writer call from an OBSERVER-mode mutation sets `observer_mutation=TRUE` on the audit row. Test via T027.
- [ ] T095 [P] Add CLI flags to `mise run sim:run` (or the underlying entry point): `--player-mode {campaign,observer}` and `--player-faction FAC_*` per quickstart.md §7

### Performance and determinism gates

- [ ] T096 [P] Write scenario test `tests/scenario/balkanization/test_wallclock_budget.py` per SC-014 + SC-015: benchmark the three new Systems combined; assert ≤5% of spec-069 canonical-run budget at steady state, ≤15% during fracture spikes
- [ ] T097 [P] Write integration test `tests/integration/balkanization/test_determinism_replay.py` per SC-011 + FR-044: run the Detroit seed twice with same seed; assert byte-identical output (per spec-069 determinism gate). Cover both CAMPAIGN-mode-with-recorded-actions and OBSERVER-mode-with-recorded-mutations per FR-050.
- [ ] T098 Hunt and fix any nondeterminism revealed by T097: common culprits are dict/set iteration order in `FactionInfluenceSystem` (use sorted iteration), unstable BFS frontier order in contiguity check (use sorted hex IDs at each frontier level)

### Constitutional + ai-docs maintenance

- [ ] T099 [P] Create ADR `ai-docs/decisions/ADR048_balkanization_political_topology.yaml` capturing: the 5 spec-070 clarifications (Q1 player modes, Q2 fracture trigger, Q3 budget, Q4 GameOutcome mapping, Q5 hard-start), the 5 research decisions (R-001 multipliers, R-002 Canadian Sovereign, R-003 partition placement, R-004 contiguity resolution, R-005 audit schema), the constitutional gates passed (I.20 realisation, IV.1 satisfied, I.18 v2 contribution), and the resulting subsystem-ownership claim per II.11
- [ ] T100 [P] Update `ai-docs/state.yaml`: increment test counts; add the new balkanization subsystem; flag the III.4 follow-up (MIT Election Lab catalog amendment as a v2.6.4 PATCH proposal)
- [ ] T101 [P] Update `ai-docs/architecture.yaml`: register the 3 new Systems in the pipeline order documentation; register the balkanization subsystem
- [ ] T102 [P] Update `ai-docs/anti-patterns.yaml` if needed — document the VIII.2 "Workers' Congress IS the trap" framing so future readers don't repair it

### Docstrings (per babylon/CLAUDE.md Sphinx-compatible policy)

- [ ] T103 [P] Add Sphinx-compatible (RST) docstrings to `src/babylon/models/entities/political_faction.py` and `src/babylon/models/entities/sovereign.py` per babylon/CLAUDE.md §Docstring Standards
- [ ] T104 [P] Add Sphinx-compatible docstrings to `src/babylon/formulas/balkanization.py` (all functions) with doctest examples that pass under `mise run test:doctest`
- [ ] T105 [P] Add Sphinx-compatible docstrings to the three new Systems (`faction_influence.py`, `sovereignty.py`, `collapse_transition.py`)
- [ ] T106 [P] Create `docs/reference/balkanization.rst` per babylon/CLAUDE.md "Maintainability Refactoring Pattern" — moves rich theory + LaTeX formulas + extended examples out of function docstrings into Sphinx-rendered RST. References balkanization-spec.yaml v1.2.0 theory section.

### Quickstart validation

- [ ] T107 Run `mise run check` and verify lint + format + typecheck + unit tests all pass for the new module tree
- [ ] T108 Run `mise run test:all` (excluding slow scenario tests) and verify everything green
- [ ] T109 Run the scenario gate: `mise run test:scenario -- -k "test_five_endgames_reachable"` and verify SC-001 + SC-002 satisfied
- [ ] T110 Execute every step in `specs/070-balkanization/quickstart.md` § 1-8 manually; resolve any drift between docs and implementation; commit any fixes

---

## Phase 7b: Post-Analyze Remediation (2026-05-18)

**Purpose**: New tasks added during the `/speckit.analyze` remediation pass to address findings C1, C3, C5, C8, and F1. T029, T035–T037 were also modified in-place to address C2 and C7 (no new task IDs needed).

### INFLUENCES seeding pipeline (C1 — addresses FR-039 coverage gap)

- [ ] T111 [P] Create `src/babylon/data/game/balkanization/__init__.py` `load_seed_influences() -> list[InfluencesEdgeSeed]` loader. Validate against `contracts/seed_influences.schema.json`. RED+GREEN with `tests/unit/balkanization/test_seed_influences_loader.py`.
- [x] T112 Create `src/babylon/data/game/balkanization/compute_seed_influences.py` — proxy-data computation pipeline producing `seed_influences.json` per FR-039 + data-model.md §8: (a) compute QCEW union-employment-share per county-year (`own_code='3'` + historically-unionized NAICS filter), prorate to res-7 hexes via LODES residential density → FAC_WORKERS_CONGRESS edges (`support_type=LABOR`); (b) intersect Natural Earth AIANNH polygons with res-7 hexes → FAC_DECOLONIAL edges (`support_type=IDEOLOGICAL`); (c) load presidential-election Republican vote share per county (MIT Election Lab if available, Census Bureau fixture otherwise), prorate to res-7 hexes → FAC_RESTORATIONIST edges (`support_type=ELECTORAL`); (d) compute complement → FAC_LIBERAL_IMPERIAL edges clamped to `liberal_imperial_influence_cap` (`support_type=IDEOLOGICAL`). Output validates against `contracts/seed_influences.schema.json`. Deterministic given upstream data + seed RNG. *(Done; P25 U6/ADR132 re-keyed the output to county FIPS per Amendment U — the LODES proration steps dissolve at county grain — and the electoral component is REAL MIT Election Lab 2024 data, ADR049 ratified.)*
- [ ] T113 Extend `src/babylon/persistence/postgres_initialization.py` (after T032 `load_seed_factions()` + `load_seed_sovereigns()` calls) to invoke `load_seed_influences()` and INSERT into `runtime_influences_edges`. Update T026 audit writer to record the seed insertions with `tick=0` and `operation='CREATE'`.

### Initial-state coverage invariant (C8 — addresses SC-017)

- [ ] T114 [P] Write integration test `tests/integration/balkanization/test_seed_coverage_invariant.py` per SC-017: after db-init, assert for every in-scope Detroit-tri-county Territory `t`: `EXISTS (INFLUENCES.* WHERE territory_id=t AND influence_level > 0) OR EXISTS (CLAIMS WHERE sovereign_id='SOV_EXTERIOR_NULL' AND territory_id=t)`. Fail loudly if any Territory is both un-influenced AND un-claimed at the start of tick 1.

### Observability projections (C3 — addresses FR-036, FR-037, SC-007, SC-013)

- [ ] T115 [P] Write tests `tests/unit/balkanization/test_observability_projections.py` for `observe_sovereign(graph, sovereign_id, horizon_ticks)` and `observe_territory(graph, territory_id)` per FR-051: verify `SovereignProjection` / `TerritoryProjection` are frozen Pydantic models with the documented fields, return deterministic snapshots, are read-only on the graph, and pass through the existing `SimulationObserver` protocol (RED)
- [ ] T116 Create `src/babylon/engine/observers/balkanization_projections.py` with `SovereignProjection` + `TerritoryProjection` Pydantic frozen models and `observe_sovereign()` + `observe_territory()` functions per FR-051. `SovereignProjection.projected_habitability` extrapolates current `metabolic_impact` over `horizon_ticks` (default 20 from BalkanizationDefines) assuming no policy change. Re-export via `engine/observers/__init__.py`. (GREEN for T115; satisfies FR-036 + FR-037; SC-007 + SC-013 verifiable as integration tests against the projection contract — UI rendering deferred to spec-042 / spec-085.)

### Cross-divide solidarity test (C5 — addresses FR-031a, SC-016)

- [ ] T117 [P] [US3] Write test `tests/integration/balkanization/test_cross_divide_solidarity_gate.py` per FR-031a + SC-016: construct two scripted scenarios — both satisfy ABOLISH-Sovereign-majority + extraction_policy=CEASE + habitability-slope ≥ 0 + percolation + class_consciousness; scenario A has < `revolutionary_victory_min_cross_divide_solidarity_edges` cross-divide SOLIDARITY edges (must route to RED_OGV); scenario B has ≥ threshold cross-divide SOLIDARITY edges (must route to REVOLUTIONARY_VICTORY with TRUE LIBERATION framing). Extend `EndgameDetector` REVOLUTIONARY_VICTORY predicate (T069) implementation to check the cross-divide SOLIDARITY edge count against `BalkanizationDefines.revolutionary_victory_min_cross_divide_solidarity_edges` (default 5).

### Catalog amendment proposal (F1 — addresses III.4 PASS-WITH-FOLLOW-UP)

- [x] T118 [P] Draft `.specify/memory/data-catalog.yaml` v2.6.4 PATCH proposal adding MIT Election Lab county-presidential (1976–2020) dataset as a Fixture-class entry under Federal Demographic. Record as a separate proposal file in `ai-docs/decisions/` (e.g., `ADR049_election_lab_catalog_addition.yaml`) rather than directly mutating the canonical data-catalog.yaml (constitutional X.1 amendment procedure requires a separate ratification step). T112's compute pipeline gracefully degrades to the Census Bureau fixture until the amendment lands. *(Ratified 2026-07-22, P25 U6/ADR132: landed as the flat MIT_Election_Lab entry in data-catalog.yaml v2.7.2 — the .specify path and v2.6.4 framing were stale; see ADR049's ratification_record.)*

**Phase 7b checkpoint**: All 11 `/speckit.analyze` findings cleared. spec-070 ready for `/speckit.implement`.

**Final checkpoint**: All 4 user stories work independently and integrated. All 60 FRs implemented (55 original + FR-031a + FR-040a + FR-040b + FR-051 + FR-052). All 17 SCs verified empirically (15 original + SC-016 + SC-017). ai-docs synced. Quickstart manually validated.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies. Starts immediately.
- **Phase 2 (Foundational)**: Depends on Setup. **BLOCKS all user stories.** All foundational tasks must complete before any user-story phase begins.
- **Phase 3 (US1, P1) 🎯 MVP**: Depends on Phase 2.
- **Phase 4 (US2, P2)**: Depends on Phase 2 + parts of Phase 3 (specifically: `SovereigntySystem.tick()` from T042 needs to exist before T057 can extend it).
- **Phase 5 (US3, P3)**: Depends on Phase 2 + Phase 3 (CollapseTransitionSystem reads SovereigntySystem outputs).
- **Phase 6 (US4, P4)**: Depends on Phase 2 + Phase 3 + Phase 4 + Phase 5. The fracture operation extends CollapseTransitionSystem (from Phase 5); the contiguity check extends FactionInfluenceSystem (from Phase 4).
- **Phase 7 (Polish)**: Depends on completion of all desired user stories.

### User Story Dependencies

- **US1 (P1)** — Independent. No dependency on US2/US3/US4.
- **US2 (P2)** — Builds on US1 (extends `SovereigntySystem.tick()`). Cannot deliver fully without US1's machinery, but US1's MVP works alone without US2.
- **US3 (P3)** — Builds on US1 + US2 (collapse triggers re-running of US2's winning-faction resolution).
- **US4 (P4)** — Builds on all of US1 + US2 + US3.

This is NOT pure-parallel-stories; spec-070 is fundamentally layered. The plan still benefits from MVP-first: User Story 1 alone is shippable as "Sovereigns drive habitability" — a 7-task-Phase-3 increment that demonstrates the core mechanic in isolation.

### Within Each User Story

- **Tests FIRST** (RED phase per `babylon/CLAUDE.md` TDD discipline). Tests must fail before implementation begins.
- **Models → Formulas → Systems → Integration**. Pydantic models land first, then derive-helpers, then the System that orchestrates them.
- **Registration LAST**. Adding a new System to `_DEFAULT_SYSTEMS` should be the final step in each user story phase — once the System's logic passes its unit tests, register it in the pipeline and run integration tests.

### Parallel Opportunities

- **Foundational phase**:
  - Enum module (T004) + enum test (T005) + enum re-exports (T009) can all run in parallel (different files).
  - Defines test (T010) + Defines impl (T011) can run in parallel with the enum work.
  - Entity tests (T015, T016) can run in parallel with each other.
  - Entity impls (T017, T018) can run in parallel.
  - Seed JSON files (T028, T029) can run in parallel.
  - Event payload models (T033) + payload tests (T034) can run in parallel with other foundational work.
- **User Story 3 tests** (T061–T065) can all run in parallel — they target different predicates in different files.
- **Polish phase** has many [P] opportunities: docstrings (T103–T106), ai-docs updates (T099–T102), CLI flags (T095), can all run in parallel.

### Dependencies-graph summary

```text
Setup (T001–T003)
        ↓
Foundational (T004–T037)
  ├─ Enums (T004–T009)
  ├─ Defines (T010, T011)
  ├─ Formulas: metabolic_impact, derive_extraction_policy, derive_default_multipliers (T012–T014, T035–T037)
  ├─ Entities (T015–T019)
  ├─ GraphProtocol extensions (T020–T023)
  ├─ Persistence (T024–T027)
  ├─ Seed data + loaders (T028–T032)
  └─ Event payloads (T033, T034)
        ↓
US1 (T038–T047)   [MVP — shippable as standalone after this]
        ↓
US2 (T048–T058)
        ↓
US3 (T059–T077)
        ↓
US4 (T078–T090)
        ↓
Polish (T091–T110)
        ↓
Post-analyze remediation (T111–T118)  # 2026-05-18 added; addresses /speckit.analyze findings
```

---

## Parallel Example: User Story 3 Tests

```bash
# All RED-phase tests for User Story 3 can be written in parallel (different files):
Task: "test_collapse_transition_system.py — collapse predicates + 5-step partition"
Task: "test_collapse_transition_pipeline.py — integration: legitimacy=0 → SOVEREIGN_COLLAPSE → TERRITORY_TRANSITION per claim"
Task: "test_endgame_revolutionary_victory_augmented.py — augmented predicate with colonial-stance gate"
Task: "test_endgame_fascist_consolidation_augmented.py — second political-violence route"
Task: "test_endgame_red_ogv.py — IGNORE-majority + class-tension-down + habitability-declining"
Task: "test_endgame_fragmented_collapse.py — no-majority + ≥3 sovereigns + insurgent/occupation/emergency"
Task: "test_endgame_priority_order.py — RED_OGV → FRAGMENTED_COLLAPSE → ECOLOGICAL → FASCIST → REV_VICTORY"
Task: "test_endgame_predicates.py — integration: each outcome reachable + exactly-one-per-run"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. **Phase 1 + Phase 2**: Setup + Foundational (T001–T037). ~25–35 hours.
2. **Phase 3**: User Story 1 (T038–T047). ~12–18 hours.
3. **STOP and VALIDATE**: Run integration test T040; manually verify US1 quickstart §3 + §4. Habitability moves under INTENSIFY; reverses under CEASE.
4. **MVP shipped**: spec-070 P1 is "Sovereigns drive habitability". Demo-able as a minimal slice.

### Incremental Delivery

| Increment | Tasks | Estimated Effort | Demo |
|---|---|---|---|
| 1. Foundation + US1 | T001–T047 | ~37–53 hrs | "Single Sovereign + ExtractionPolicy + habitability changes" |
| 2. + US2 | T048–T058 | +14–20 hrs | "Three Factions compete; winning Faction installs Sovereign" |
| 3. + US3 | T059–T077 | +24–34 hrs | "Five endgames reachable + RED_OGV pedagogy works" |
| 4. + US4 | T078–T090 | +18–28 hrs | "Active secession + O(1) fracture + civil war" |
| 5. + Polish | T091–T110 | +10–15 hrs | "Determinism gate green; ai-docs synced; quickstart validated" |
| 6. + Post-analyze remediation | T111–T118 | +8–12 hrs | "INFLUENCES proxy seeding, observability projections, cross-divide-solidarity gate, MIT Election Lab catalog amendment proposal" |

Cumulative: ~103–150 hrs, consistent with the audit's ~140–180h estimate.

### Parallel Team Strategy

If multiple developers are available after Phase 2 completes:

- **Developer A**: US1 → US2 (sequential, since US2 extends US1's system)
- **Developer B**: US3 (depends on US1 + US2, but Phase 5 has 8 independent test files T061–T065 + 2 scenario tests T076–T077 that can start in parallel once US2 is done)
- **Developer C**: GraphProtocol extension tests (T020–T023) + Hypothesis property scaffolding in foundational phase; can start US4 contiguity work (T078, T082) earlier as long as it integrates after US2/US3 land

Note: there's no honest parallel split for US1 vs US2 (US2 extends US1's system in T057) or US3 vs US4 (US4 extends US3's system in T084). spec-070 is layered, not laminar.

---

## Notes

- All `[P]` tasks operate on different files with no incomplete-dependency conflicts.
- Tests use the existing project markers per `babylon/CLAUDE.md` Test Reports section. Unit tests run via `mise run test:unit`; integration via `mise run test:int`; scenario via `mise run test:scenario`.
- All multipliers and thresholds funnel through `BalkanizationDefines` per research.md R-001; no magic numbers in System logic.
- Per `babylon/CLAUDE.md` "Commit after each unit of work": commit after each task or logical task group (e.g., commit after all foundational enums; commit after US1 completes).
- Per `babylon/CLAUDE.md` "Pydantic First": all new game objects are `pydantic.BaseModel` subclasses with `model_config = ConfigDict(frozen=True)`. No raw dicts.
- Per Constitution III.7 + spec-069 byte-identical-determinism gate: every new System must be deterministic-by-seed. Use sorted iteration for any dict/set traversal that affects emitted events or state mutations.
- Per Constitution II.11: the `balkanization` subsystem owns its tables. Cross-subsystem reads (e.g., Territory.habitability) MUST go through GraphProtocol, never direct table access.
- Per Constitution V Verb Atomicity: OBSERVER-mode mutations (FR-049) MUST each be atomic graph operations — one edge or one node per mutation — with the audit flag set.
