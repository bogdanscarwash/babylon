---
type: Explanation
title: "Quickstart — Babylon Wiki"
description: "Entry point for the Babylon wiki. Orients humans and agents to the three namespaces (Glossary, State, Flavor), the core mantra, current project status, and navigation to all major pages."
tags: [entrypoint, orientation, babylon]
timestamp: "2026-07-30T00:00:00Z"
---

# Quickstart — Babylon Wiki

> **Mantra:** **Graph + Math = History**
>
> Babylon is a deterministic geopolitical simulation engine modeling the collapse of American hegemony through **MLM-TW theory** (Marxist-Leninist-Maoist Third Worldist) and a **Lawverian topological algebra**. The engine runs locally without external servers using the **Embedded Trinity** architecture: Ledger (PostgreSQL/SQLite), Topology (rustworkx/BabylonGraph), Archive (pgvector).

---

## How to Use This Wiki

This wiki is organized into **three namespaces** (per the Director's architecture directive, issue #335). **Never mix namespaces in one page.**

| Namespace | Purpose | Update Cadence | Audience |
|-----------|---------|----------------|----------|
| **[Glossary](/openwiki/glossary/)** | Stable concept definitions — dialectic `D=(A,Ā,w,T,σ)`, Imperial Rent Φ, Survival Calculus, hyperedge, fuel, ceremony… | Only when a ruling/ADR moves a definition | Both (definitions are the contract) |
| **[State](/openwiki/state/)** | What is true of the codebase **NOW** — architecture, systems order, gates, commands, data pipeline | Regenerated aggressively on merge to `dev` | **Agents primarily** (token-efficient context), humans secondarily |
| **[Flavor](/openwiki/flavor/)** | Narrative/theory exposition — cites the ruled ideological line | Only on Director rulings | **Humans primarily** (confidence-building, orientation) |

**Navigation rule:** Start here. Every major concept links to its canonical Glossary page. Current implementation state lives in State. Theory exposition lives in Flavor.

---

## Current Project Status (as of `786893fc` — 2026-07-29)

> **Source of truth:** [`ai/state.yaml:truth_status`](/ai/state.yaml), [`NORTH_STAR.md`](/NORTH_STAR.md), [`CONSTITUTION.md`](/CONSTITUTION.md) v3.0.0

| Epoch | Status |
|-------|--------|
| **Epoch 1: Engine** | ✅ COMPLETE (25 systems, 70 EventTypes, 55 formula functions) |
| **Epoch 2: Foundation** | ⚡ IN PROGRESS — 3NF schema, census loaders, H3 geography complete; schema integration, LODES/freight, ideological cartography partial |
| **Epoch 3: Game** | ⚡ IN PROGRESS — 17 specs (011–066), Balkanization (070) shipped, 3.7/3.9/3.10 in flight |
| **Epoch 4: Platform** | 📋 VISION — DuckDB unification, Postgres 16 + pgvector (spec-037/ADR030) |

### Active Program: **Program 27 — Refoundation (Rust Kernel + BSL)**

- **Amendment AE ratified (v3.0.0, ADR172)**: Rust is the engine language; Python survives as data-build pipeline, out-of-process AI observer, and CLI periphery. Python engine freezes at `p27-python-freeze`.
- **BSL (Babylon Scripting Language)** is the **one additive formal construct** — expresses the closed algebra, mints no new mathematics.
- **Amendment D ruled NATIVE HYPEREDGE**: Hyperedges are first-class in `babylon-graph`'s exposed model; Levi/incidence is internal storage only.
- **Clause (xi)**: `ratty` + `Ratatui` are **required renderers** for in-game topology, hypergraph structures, and value-flow Sankey diagrams.
- **Theory ruling (same session)**: **No imposed functional forms** (sigmoids included). Curve shapes must **emerge** from P(revolution)/P(acquiescence) and the Lawverian algebra.
- **Phase 0 engineering complete** (16/16 tasks merged). Phase 1 blocked on: (1) `phi_hour` live regression fix, (2) RNG seed-threading gap, (3) v3.0.0 ratification merge.

### Active Remediation: **Loud Machine Program**

The July 7 playtest found the core loop broken; the holistic review found silent failures behind it. The **Loud Machine remediation** fixes everything found and wires the system so failures are loud instead of silent.

- **P0s resolved**: Schema parity crash, verb payloads rejected, verb targets were fixture IDs, tick-resolve datetime 500, map renders zero features, verbs were engine-side no-ops.
- **Gates live**: C.1 roundtrip, C.2(a) determinism A/B, C.3 migration idempotency, C.4 CI Postgres, C.5 Playwright, C.9 test-infra re-arm, C.13 resolving watchdog.
- **In flight**: Storage gates A/B/C, degradation envelope + stub visibility, nightly A/B, deptry, doc-ref linter, budget gates, wiring audit.

---

## Quick Navigation

### 📚 Glossary (Stable Definitions)
| Page | Core Concepts |
|------|---------------|
| [Core Concepts](/openwiki/glossary/core-concepts.md) | Dialectic `D=(A,Ā,w,T,σ)`, Imperial Rent Φ, Survival Calculus, Lawverian algebra, Hyperedge, Fuel, Ceremony |
| [Architecture Concepts](/openwiki/glossary/architecture-concepts.md) | Embedded Trinity, BabylonGraph, WorldState, GraphProtocol, ServiceContainer, Systems order |
| [Game Concepts](/openwiki/glossary/game-concepts.md) | Franchise Model (Agents vs Infrastructure), Fog of War (Water/Mud/Desert), 9 Verbs, Five Outcomes |

### ⚙️ State (Current Implementation)
| Page | What You'll Find |
|------|------------------|
| [Architecture Snapshot](/openwiki/state/architecture.md) | 34 systems, Embedded Trinity status, client status (Rust/Ratatui = `babylon play`), Python freeze |
| [Systems Order](/openwiki/state/systems-order.md) | 34 systems in materialist-causality order (Material Base → Action → Consequences) with rationale |
| [Developer Commands](/openwiki/state/commands.md) | `mise run setup`, `mise run check`, `mise run sim:report`, `mise run reference:python-smoke`, analysis commands |
| [CI Gates & Sentinels](/openwiki/state/gates.md) | C.1–C.13 gates, sentinel doctrine, ceremony gates, coverage requirements |
| [Data Pipeline](/openwiki/state/data-pipeline.md) | Parquet canonical sources (ADR098), `marxist-data-3NF.sqlite` as build product, `tools/build_reference_db.py` |

### 🎭 Flavor (Theory & Narrative)
| Page | What You'll Find |
|------|------------------|
| [The Ruled Theory Line](/openwiki/flavor/theory-line.md) | MLM-TW commitments, five outcomes, no imposed sigmoids (ADR172 ruling 5), Director sole authority |
| [Five Canonical Outcomes](/openwiki/flavor/five-outcomes.md) | REVOLUTIONARY_VICTORY, ECOLOGICAL_COLLAPSE, FASCIST_CONSOLIDATION, RED_OGV, FRAGMENTED_COLLAPSE |
| [Narrative Architecture](/openwiki/flavor/narrative.md) | NarrativeDirector (observer, read-only), pgvector RAG, dialectical prompt builder, wire/analysis registers |

---

## Essential Commands (Human Onboarding)

```bash
# One-shot fresh-clone bootstrap: toolchain + deps + hooks + tuned Postgres 16 + schema
mise trust
mise run setup

# Verify the frozen Python reference with a self-contained smoke test
mise run reference:python-smoke

# Fast CI gate (lint + format + typecheck + unit tests)
mise run check

# Start the web game (Django :8000 + Vite :5173 as background daemons)
mise run web:dev

# Run the terminal game (Rust/Ratatui client — the ONLY client since M7 cutover)
babylon play
```

> **Requirements:** [mise](https://mise.jdx.dev) (provisions Python 3.12 + uv) and Docker (for tuned Postgres 16 compose stack). New to the project? See [SETUP_GUIDE.md](/SETUP_GUIDE.md) for OS-by-OS walkthrough.

---

## Key Architectural Invariants (Do Not Contradict)

| Invariant | Source | Since |
|-----------|--------|-------|
| **Rust is the engine language** | Amendment AE, v3.0.0, ADR172 | 2026-07-29 |
| **BSL is the one additive formal construct** | Amendment AE, NORTH_STAR.md §0 | 2026-07-29 |
| **Hyperedges are first-class in babylon-graph** | Amendment D (rulings D-1..D-7) | 2026-07-28 |
| **ratty + Ratatui are required renderers** | Amendment AE clause (xi) | 2026-07-29 |
| **AI observes and narrates only; engine adjudicates math** | Constitution II.5, Amendment V/Y | 2026-07-27 |
| **Every tick is deterministic (seeded RNG, tick-derived timestamps)** | Constitution III.7 | 2026-01-05 |
| **Python engine freezes at `p27-python-freeze`** | Amendment AE, ADR172 | 2026-07-29 |
| **No imposed sigmoids — curve shapes emerge** | ADR172 ruling 5, NORTH_STAR.md §0 | 2026-07-29 |

---

## Where to Go Next

| If you are... | Start here |
|---------------|------------|
| **New human contributor** | [Glossary: Core Concepts](/openwiki/glossary/core-concepts.md) → [State: Architecture Snapshot](/openwiki/state/architecture.md) → [Flavor: Theory Line](/openwiki/flavor/theory-line.md) |
| **AI agent needing context** | [State: Architecture Snapshot](/openwiki/state/architecture.md) → [State: Systems Order](/openwiki/state/systems-order.md) → [Glossary: Architecture Concepts](/openwiki/glossary/architecture-concepts.md) |
| **Debugging a system** | [State: Systems Order](/openwiki/state/systems-order.md) → [State: Commands](/openwiki/state/commands.md) → [State: Gates](/openwiki/state/gates.md) |
| **Understanding the math** | [Glossary: Core Concepts](/openwiki/glossary/core-concepts.md) → [Flavor: Theory Line](/openwiki/flavor/theory-line.md) → `ai/THE_FORMALISM.md` |
| **Working on the web game** | [State: Architecture Snapshot](/openwiki/state/architecture.md) (client status) → `web/`, `src/frontend/` (legacy per Amendment V) |

---

## Backlog (Deferred Pages)

| Area | Source Anchor | Reason Deferred |
|------|---------------|-----------------|
| Formula System (~56 functions / 17 modules) | `src/babylon/formulas/`, `ai/formulas-spec.yaml` | Not yet blocking; can be added when formula work resumes |
| Web/Django Bridge Architecture | `web/`, `src/frontend/` | Legacy per Amendment V; failures don't gate work |
| Simulation Lab (trace, sweep, Monte Carlo, Morris/Sobol/Optuna) | `tools/` | Specialized tooling; add when tuning work is active |
| Lawverian Dialectics / Contradiction Algebra | `ai/state-ai-algorithm.yaml`, `docs/concepts/dialectical-field-theory.rst` | Theoretical depth; add when dialectics work is active |

---

*Generated against commit `786893fc9f25954312fb654bcb67fb650bbc3820` (2026-07-29). See per-page provenance for source files.*
