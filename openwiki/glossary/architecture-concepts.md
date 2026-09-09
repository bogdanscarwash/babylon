---
type: Reference
title: "Glossary — Architecture Concepts"
description: "Stable definitions of the Babylon system architecture primitives: Embedded Trinity (Ledger, Topology, Archive), BabylonGraph, WorldState, GraphProtocol, ServiceContainer, Systems order, Franchise Model."
tags: [glossary, architecture, embedded-trinity, topology]
timestamp: "2026-07-30T00:00:00Z"
---

# Glossary — Architecture Concepts

> **Namespace:** `Glossary` — Stable concept definitions. Updated only when a ruling/ADR moves a definition.
> **Source of truth:** `ai/architecture.yaml`, `NORTH_STAR.md` §1–§2, `src/babylon/engine/simulation_engine.py`, `CONSTITUTION.md` Articles I–III.

---

## The Embedded Trinity

> **Definition:** The three-layer local system architecture with **no external servers**. Each layer has a distinct purpose, technology, and data flow. **Principle:** State is pure data; the engine is pure transformation; they never mix.
>
> **Source:** `NORTH_STAR.md` §1; `ai/architecture.yaml` `core_architecture`; `README.md` "Architecture Principle".

<!-- openwiki: mermaid parse failed and this diagram was converted to a text fence so it does not break rendering. Fix the diagram source and restore the mermaid fence. Parser error: Heuristic: an unescaped angle bracket inside a label breaks rendering; rephrase the label. -->
```text
flowchart TB
    subgraph LEDGER["THE LEDGER — Rigid Material State"]
        SQLITE[(SQLite Reference DB<br/>marxist-data-3NF.sqlite)]
        POSTGRES[(PostgreSQL Runtime<br/>spec-037)]
        PYDANTIC[Pydantic Models<br/>Validation + Hydration]
    end
    
    subgraph TOPOLOGY["THE TOPOLOGY — Fluid Relational State"]
        BABYLON_GRAPH[BabylonGraph<br/>rustworkx PyDiGraph core]
        GRAPH_PROTOCOL[GraphProtocol<br/>Interface abstraction]
        WORLD_STATE[WorldState<br/>to_graph() / from_graph()]
    end
    
    subgraph ARCHIVE["THE ARCHIVE — Semantic History"]
        PGVECTOR[pgvector in Postgres<br/>spec-037, replaced ChromaDB]
        NARRATOR[NarrativeDirector<br/>Observer Pattern]
        RAG[RAG Pipeline<br/>Embedding Retrieval]
    end
    
    SQLITE -->|Hydrate| BABYLON_GRAPH
    POSTGRES -->|Hydrate| BABYLON_GRAPH
    PYDANTIC -->|Validate| WORLD_STATE
    WORLD_STATE -->|to_graph()| BABYLON_GRAPH
    BABYLON_GRAPH -->|from_graph()| WORLD_STATE
    WORLD_STATE -->|Dehydrate| POSTGRES
    BABYLON_GRAPH -->|Events| NARRATOR
    NARRATOR -->|Query| PGVECTOR
    RAG -->|Retrieve| PGVECTOR
```

### Layer Details

| Layer | Purpose | Technology | Key Files | Data Flow |
|-------|---------|------------|-----------|-----------|
| **Ledger** | Rigid, material state (persistent storage) | SQLite (reference, read-only) + PostgreSQL (runtime) + Pydantic | `src/babylon/persistence/`, `src/babylon/reference/`, `data/sqlite/marxist-data-3NF.sqlite` | **Hydration Pattern**: Load at startup → persist on save → **NO DB I/O during tick** |
| **Topology** | Fluid, relational state (hot computation) | rustworkx via `BabylonGraph` (Amendment L / ADR052) | `src/babylon/topology/graph.py`, `src/babylon/models/world_state.py` | Hydrated from Ledger → mutated during tick → dehydrated back |
| **Archive** | Semantic history for AI narrative | pgvector in Postgres (spec-037) | `src/babylon/intelligence/rag/`, `src/babylon/persistence/pgvector_store.py` | AI queries for context → generates narrative from state changes |

---

## BabylonGraph (Topology Substrate)

> **Definition:** The graph substrate implementation. **rustworkx `PyDiGraph` core** + insertion-ordered payload/adjacency mirrors reproducing NetworkX's iteration contract (determinism, byte-identical baselines). **Both** the `GraphProtocol` implementation **and** the nx-compat authoring API.
>
> **Constitutional basis:** Amendment L (2026-07-03), ADR052. NetworkXAdapter was deleted.
> **Source:** `ai/architecture.yaml` `topology`; `src/babylon/topology/graph.py`; `docs/reference/determinism-contract.rst`.

### Key Properties
| Property | Value |
|----------|-------|
| Backing | rustworkx `PyDiGraph` |
| Mirrors | Insertion-ordered payload/adjacency (dict-like) |
| Node types | `social_class`, `territory`, `organization`, `institution`, `sovereign`, `hex`, `industry`, `key_figure` |
| Edge types | `EXPLOITATION`, `SOLIDARITY`, `WAGES`, `TRIBUTE`, `TENANCY`, `ADJACENCY`, `REPRESSION` |
| Determinism | Byte-identical baselines (III.7) |
| Hyperedges | **Native first-class** (Amendment D rulings D-1..D-7); Levi/incidence internal only |

### GraphProtocol Interface
The architecture defines a **GraphProtocol interface**, NOT a specific graph library. The substrate is an implementation detail. Backends can be swapped without changing any System code.
- **Current:** `BabylonGraph` (rustworkx) — `src/babylon/engine/graph.py`
- **Future:** `ColumnarAdapter` (DuckDB + DuckPGQ) — Epoch 3+ (planned)

---

## WorldState (The Immutable Snapshot)

> **Definition:** The frozen Pydantic model representing the complete simulation state at a tick boundary. **Immutable** — each tick produces a new `WorldState`; the old state is unchanged.
>
> **Key Methods:**
> - `to_graph()` → `BabylonGraph` (hydration for simulation)
> - `from_graph(graph)` → `WorldState` (dehydration after simulation)
>
> **Source:** `src/babylon/models/world_state.py`; `ai/architecture.yaml` `topology.contains`.

### Entity Types in WorldState
| Type | Description | Key Fields |
|------|-------------|------------|
| `SocialClass` | Class agents (Proletariat, LaborAristocracy, Bourgeoisie, etc.) | `wealth`, `ideology`, `organization`, `population`, `class_consciousness`, `national_identity`, `agitation` |
| `Territory` | Spatial substrate (Layer 0) | `profile`, `heat`, `rent_level`, `biocapacity`, `internet_capacity` |
| `Relationship` | Directed edges between entities | `edge_type`, `value_flow`, `tension`, `solidarity_strength` |
| `Organization` | Strategic actors (Player org, State, rival factions) | `war_chest`, `doctrine_tree`, `cells` |
| `Sovereign` | Political entities (nation-states, breakaway regions) | `legitimacy`, `territory_control`, `policy_axis` |

---

## ServiceContainer (Dependency Injection)

> **Definition:** The concrete DI container aggregating all services needed for a tick. **Protocol** lives in `kernel/services.py`; **implementation** in `engine/services.py`.
>
> **Services Provided:**
> - `config: SimulationConfig` (frozen, 11 parameters — only `rng_seed` per Constitution III.7)
> - `formulas: FormulaRegistry` (24 hot-swappable formulas)
> - `event_bus: EventBus` (in-memory, 100 EventTypes)
> - `database: DatabaseConnection` (SQLAlchemy 2.0, **not used during tick**)
> - `metrics: MetricsCollector` (observes derived metrics)
> - `persistence: RuntimePersistence` (Postgres/SQLite, **not used during tick**)
>
> **Source:** `src/babylon/engine/services.py`; `src/babylon/kernel/services.py`; `ai/architecture.yaml` `simulation.services`.

---

## Systems Order (Materialist Causality)

> **Definition:** The 34 modular Systems executed in **strict materialist-causality order** each tick. Order encodes: **Base before Superstructure** — biological/spatial/economic systems run before ideological/social/dialectical systems.
>
> **Source of Truth:** `src/babylon/engine/simulation_engine.py:_DEFAULT_SYSTEMS` (derived from each System's `position` ClassVar).
> **Source:** `ai/architecture.yaml` `systems_execution`; `simulation_engine.py` lines 280–405.

### Three Phases
| Phase | Position Range | Systems | Purpose |
|-------|----------------|---------|---------|
| **Material Base** | 1–13 (+ Substrate @2.5) | Vitality, Territory, Production, TickDynamics, ReserveArmy, Community, Lifecycle, Solidarity, ImperialRent, Transport, Dispossession, Decomposition, ControlRatio, Metabolism | Produce material conditions |
| **Action** | 14 | OODA | Organizations observe + act |
| **Consequences** | 15–34 | FactionInfluence, Doctrine, Survival, Struggle, Consciousness, FascistFaction, Allegiance, Electoral, Policy, Sovereignty, MarketScissors, Contradiction, ContradictionField, FieldDerivative, CollapseTransition, EdgeTransition, WealthDistribution, EpistemicHorizon | Consequences follow actions |

> **Constitutional basis:** ADR032 (Materialist Causality), Constitution III.7 (Determinism).

---

## Franchise Model (Agents vs Infrastructure)

> **Definition:** The architectural principle that **Strategic Agents** (Revolutionary Organization, State, rival factions) are **DISTINCT** from **Graph Nodes** (Cells, Territories, SocialClasses). Agents are "CPUs" that **manipulate** the "Data" (Graph) via the `GraphProtocol` interface.
>
> **Analogy:** Franchise business — Owner (Agent) manages multiple locations (Nodes); Owner is NOT a location; multiple Owners can compete for control of the same Nodes.
>
> **Source:** `ai/architecture.yaml` `franchise_model`.

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

## Dual Pipeline (Epoch 1 → 2 Bridge)

> **Definition:** Two parallel pipelines from `WorldState` to UI, preserving the Epoch 1 "God View" while adding Epoch 2 "Player View" with Fog of War.
>
> **Source:** `ai/architecture.yaml` `dual_pipeline`.

<!-- openwiki: mermaid parse failed and this diagram was converted to a text fence so it does not break rendering. Fix the diagram source and restore the mermaid fence. Parser error: Heuristic: an unescaped angle bracket inside a label breaks rendering; rephrase the label. -->
```text
flowchart LR
    ENGINE[SimulationEngine] --> WORLD[WorldState<br/>Source of Truth]
    WORLD -->|Pipeline A: Truth| GOD[Debug UI<br/>God View<br/>100% accurate, 0 latency]
    WORLD --> FOG[FogOfWarSystem] --> SHADOW[PlayerShadowState<br/>Subjective View] -->|Pipeline B: Subjectivity| PLAYER[Game UI<br/>Player View<br/>Variable accuracy, stale, masked]
```

### Pipeline A: Truth (God View)
- **Accuracy:** 100% — sees TRUE state of all attributes
- **Latency:** 0 — instant access to current tick
- **Masking:** None — no fog of war
- **Purpose:** Development, debugging, demonstration
- **Status:** Preserved exactly as-is (Epoch 1)

### Pipeline B: Subjectivity (Player View)
- **Accuracy:** Variable — depends on `Mass_Receptivity`
- **Latency:** May be stale — intel decays over time
- **Masking:** Yes — Desert territories show FALSIFIED data
- **Purpose:** Gameplay with fog of war
- **Status:** Infrastructure being built (Epoch 2)

### Intel Quality States (per Territory)
| State | Condition | Intel Confidence | Data |
|-------|-----------|------------------|------|
| **Water** | `Mass_Receptivity >= 0.8` | 0.9–1.0 | TRUE — matches WorldState |
| **Mud** | `0.2 <= Mass_Receptivity < 0.8` | 0.4–0.7 | APPROXIMATE — ±0.2 margin |
| **Desert** | `Mass_Receptivity < 0.2` | 0.1–0.3 | MASKED — may be FALSIFIED |

---

## Cross-References

| Concept | Glossary Page | State Page | Flavor Page |
|---------|---------------|------------|-------------|
| Embedded Trinity | This page | [Architecture Snapshot](/openwiki/state/architecture.md) | — |
| BabylonGraph | This page | [Architecture Snapshot](/openwiki/state/architecture.md#topology) | — |
| WorldState | This page | [Architecture Snapshot](/openwiki/state/architecture.md#worldstate) | — |
| Systems Order | This page | [Systems Order](/openwiki/state/systems-order.md) | — |
| Franchise Model | This page | [Architecture Snapshot](/openwiki/state/architecture.md#franchise-model) | — |
| Dual Pipeline | This page | [Architecture Snapshot](/openwiki/state/architecture.md#dual-pipeline) | — |

---

*Generated against commit `786893fc9f25954312fb654bcb67fb650bbc3820`. Sources: `ai/architecture.yaml` v4.1.0, `NORTH_STAR.md`, `src/babylon/engine/simulation_engine.py`, `src/babylon/topology/graph.py`, `src/babylon/models/world_state.py`, `src/babylon/engine/services.py`, `CONSTITUTION.md`.*