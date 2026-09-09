---
type: Reference
title: "Glossary — Game Concepts"
description: "Stable definitions of the game-layer concepts: Franchise Model (Agents vs Infrastructure), Fog of War (Water/Mud/Desert), the 9 Article V Verbs, PlayerShadowState, and the Five Canonical Outcomes."
tags: [glossary, game, verbs, fog-of-war, franchise-model]
timestamp: "2026-07-30T00:00:00Z"
---

# Glossary — Game Concepts

> **Namespace:** `Glossary` — Stable concept definitions. Updated only when a ruling/ADR moves a definition.
> **Source of truth:** `ai/architecture.yaml` `franchise_model`, `fog_of_war_system`, `CONSTITUTION.md` Article V, `NORTH_STAR.md` §4, `src/babylon/engine/observers/endgame_detector.py`.

---

## Franchise Model (Agents vs Infrastructure)

> **Definition:** The architectural principle that **Strategic Agents** (the Revolutionary Organization / Player, the State / AI opponent, rival factions) are **DISTINCT** from **Graph Nodes** (SocialClasses, Territories, Organizations, Institutions). Agents are "CPUs" that **manipulate** the "Data" (Graph) via the `GraphProtocol` interface.
>
> **Analogy:** Franchise business — the Owner (Agent) manages multiple locations (Nodes); the Owner is NOT a location; multiple Owners can compete for control of the same Nodes.
>
> **Source:** `ai/architecture.yaml` `franchise_model`; `NORTH_STAR.md` §2 (Stratum 0 vs Stratum 1).

### Separation
| Category | Description | Examples | Location |
|----------|-------------|----------|----------|
| **Agents** | Strategic actors with resources, goals, decision-making | Revolutionary Organization (Player), State (AI), Rival Organizations | **NOT in the graph** — external entities |
| **Infrastructure** | Material reality represented as graph nodes/edges | SocialClass nodes, Territory nodes, Edges (SOLIDARITY, EXPLOITATION, TENANCY, ADJACENCY) | **The Graph** (Topology layer) |

### Implications
- Agents query/mutate the graph via `GraphProtocol` (Epoch 2+)
- In Epoch 1, separation is conceptual (no formal Agent class yet)
- **Fog of War applies to Agents, not to the Graph** (Graph is Truth)
- Multiple Agents can have different views of the same Graph

---

## Fog of War — Water / Mud / Desert

> **Definition:** The epistemic filter that transforms `WorldState` (Truth) into `PlayerShadowState` (Subjective View) per territory. **Never modifies WorldState** — the Shadow is always a derivative view.
>
> **Source:** `ai/architecture.yaml` `fog_of_war_system`, `player_shadow_state`; `docs/concepts/ai-integration.rst`.

### Intel Quality States (per Territory)

| State | Condition | Intel Confidence | Data Quality | Description |
|-------|-----------|------------------|--------------|-------------|
| **Water** | `Mass_Receptivity >= 0.8` | 0.9–1.0 | **TRUE** — matches WorldState | Base area — masses are your eyes |
| **Mud** | `0.2 <= Mass_Receptivity < 0.8` | 0.4–0.7 | **APPROXIMATE** — ±0.2 margin of error | Contested — partial information |
| **Desert** | `Mass_Receptivity < 0.2` | 0.1–0.3 | **MASKED** — may be FALSIFIED | Hostile — locals deceive you |

### Generation Process
1. Read `WorldState` (Truth)
2. For each territory, calculate `Mass_Receptivity`
3. For each territory, calculate `Intel_Confidence`
4. For Desert territories, apply **MASKING** (generate plausible false data)
5. For Mud territories, apply **APPROXIMATION** (add noise)
6. Store result as `PlayerShadowState`

### Debug UI Integration
- **Left panel:** `WorldState` (Truth) — always accurate
- **Right panel:** `PlayerShadowState` (Shadow) — what player would see
- **Diff highlight:** Shows which values are masked/approximate

---

## The 9 Article V Verbs (Player Expressive Grammar)

> **Definition:** The nine verbs are the player's **entire expressive grammar** — every one must appear in the tutorial-BDD suite or it is a dead option (a `∂L` seam) and the gate is red.
>
> **Constitutional basis:** `CONSTITUTION.md` Article V; `NORTH_STAR.md` §4 "The grammar — everything the player touches is text".
>
> **Source:** `src/babylon/engine/actions/` (verb resolvers); `web/game/api.py` (API endpoints); `src/babylon/models/enums/events.py` (EventType enum).

| Verb | ActionType | Description | Engine Effect |
|------|------------|-------------|---------------|
| **ORGANIZE** | `ORGANIZE` | Build cells, recruit cadres | Increases `organization`, creates SOLIDARITY edges |
| **AGITATE** | `AGITATE` | Raise consciousness in a territory | Increases `agitation`, `class_consciousness` |
| **STRIKE** | `STRIKE` | Withhold labor, disrupt production | Reduces `ProductionSystem` output, increases `tension` |
| **PROTEST** | `PROTEST` | Public demonstration | Increases `heat`, affects `Mass_Receptivity` |
| **INSURRECT** | `INSURRECT` | Armed uprising | Triggers `StruggleSystem`, may cascade to `CollapseTransition` |
| **INVESTIGATE** | `INVESTIGATE` | Gather intelligence | Improves `Intel_Confidence` in target territories |
| **PROPAGANDIZE** | `PROPAGANDIZE` | Ideological warfare | Shifts `ideology`, affects `doctrine_tree` |
| **NEGOTIATE** | `NEGOTIATE` | Diplomatic/electoral engagement | Modifies `PolicyAxis`, `Allegiance` |
| **MOVE** | `MOVE` | Relocate cadres/cells | Changes territorial control, affects `TENANCY` edges |

### Verb Resolution Pipeline
```
Player issues verb
      │
      ▼
VERB_RESOLVERS (9 resolvers in engine/actions/)
      │
      ▼
OODASystem (position 14) — observes Material Base, then acts
      │
      ▼
Action.params → effects applied to graph
      │
      ▼
Consequences phase (positions 15–34) — Survival, Struggle, Consciousness, etc.
```

> **Note:** The legacy web bridge (`web/game/engine_bridge.py`) maps 9 verbs to the engine. The `ActionType` enum has 25 values but only 9 are wired to resolvers (16 silently returned 0.0 before remediation — fixed in P0 #8, merge `9f6f244e`).

---

## PlayerShadowState (Subjective View)

> **Definition:** The player's subjective snapshot of the world, derived from `WorldState` via the `FogOfWarSystem`. **Always generated FROM WorldState, never independent.**
>
> **Characteristics:**
> - **Derivative:** Always generated from `WorldState`
> - **Mutable:** Can be stale (intel decay), approximate (Mud), or FALSE (Desert)
> - **Per-territory:** Each territory has its own `Intel_Confidence`
>
> **Source:** `ai/architecture.yaml` `player_shadow_state`.

### Data Flow
```
WorldState (Truth) ──► FogOfWarSystem ──► PlayerShadowState (Shadow) ──► Game UI
       │                    │                      │
       │                    │                      └─ Variable accuracy, stale, masked
       │                    └─ Calculates Mass_Receptivity, Intel_Confidence per territory
       └─ Immutable, accurate, canonical
```

---

## Five Canonical Outcomes (Terminal States)

> **Definition:** The five exhaustive, mutually exclusive endgame outcomes detected by `EndgameDetector`. Each represents a distinct historical trajectory of the collapse.
>
> **Source:** `src/babylon/engine/observers/endgame_detector.py`; `docs/concepts/terminal-crisis.rst`; `docs/concepts/warlord-trajectory.rst`; `CONSTITUTION.md` II.4.

| Outcome | Trigger Condition | Theoretical Basis |
|---------|-------------------|-------------------|
| **REVOLUTIONARY_VICTORY** | Revolutionary consciousness percolates to national sovereignty; `P(S\|R) > P(S\|A)` sustained nationally | Successful proletarian revolution; dual power → workers' state |
| **ECOLOGICAL_COLLAPSE** | Metabolic rift overshoot `O = C/B > 1` sustained; biocapacity exhausted | Metabolic rift theory; `ΔB = R − (E × η)` driven negative irreversibly |
| **FASCIST_CONSOLIDATION** | Fascist faction captures state apparatus via bifurcation (`+1` ideology route); repression overwhelms organization | Fascist pull from agitation without SOLIDARITY; `ControlRatio` terminal decision |
| **RED_OGV** (Old Guard Victory) | Reformist electoral path liquidates revolutionary potential; `DoctrineTree` reformist trunk absorbs all tag movement | Electoral cretinism; officeholder capture; `PracticeVariable` measured practice flows only through reformist channel |
| **FRAGMENTED_COLLAPSE** | Balkanization → warlord trajectory → no coherent successor sovereign | `SovereigntySystem` collapse; secession cascades; `warlord-trajectory` branching |

### Detection
- `EndgameDetector` implements `SimulationObserver` (read-only)
- Runs at end of Consequences phase (after `CollapseTransitionSystem`)
- Emits `EndgameEvent` with outcome type
- Triggers session termination with narrative epilogue

---

## Cross-References

| Concept | Glossary Page | State Page | Flavor Page |
|---------|---------------|------------|-------------|
| Franchise Model | This page | [Architecture Snapshot](/openwiki/state/architecture.md#franchise-model) | — |
| Fog of War | This page | [Architecture Snapshot](/openwiki/state/architecture.md#dual-pipeline) | — |
| 9 Verbs | This page | [Systems Order](/openwiki/state/systems-order.md#oodasystem) | [Theory Line](/openwiki/flavor/theory-line.md#verbs) |
| PlayerShadowState | This page | [Architecture Snapshot](/openwiki/state/architecture.md#playershadowstate) | — |
| Five Outcomes | This page | [Architecture Snapshot](/openwiki/state/architecture.md#endgame-detector) | [Five Outcomes](/openwiki/flavor/five-outcomes.md) |

---

*Generated against commit `786893fc9f25954312fb654bcb67fb650bbc3820`. Sources: `ai/architecture.yaml`, `CONSTITUTION.md` v3.0.0 Article V, `NORTH_STAR.md` §2–§4, `src/babylon/engine/observers/endgame_detector.py`, `src/babylon/engine/actions/`, `src/babylon/engine/systems/ooda.py`.*