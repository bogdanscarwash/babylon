# Babylon AI Documentation Context

Machine-readable documentation for LLM-assisted development.

This directory helps AI assistants understand context, use correct terminology, follow established patterns, avoid known pitfalls, and make decisions aligned with project philosophy.

---

## ACTIVE SPECS (The Truth)

| File | Purpose |
|------|---------|
| `../reports/aidocs-vs-code-audit-2026-05-16.md` | **LIVING ROADMAP** (owner decision 2026-07-02) — epoch-vs-code audit + 27-spec full-vision catalog in 7 waves. Spec numbers in the catalog are advisory; actual numbers are first-come at spec-creation time (086/097 were consumed by the QCEW data-quality track). |
| `state.yaml` | Current project state, test counts, sprint status |
| `epochs/overview.md` | Historical vision: Four Epochs (Engine, Foundation, Game, Platform). Status tables frozen ~Jan 2026 — see the audit report for what actually shipped. |
| `epochs/epoch1-complete.md` | Epoch 1 completion record (counts as of 2026-01-05; since grown: 25 systems, 70 EventTypes, 55 formula functions) |
| `epochs/epoch2/` | Epoch 2: The Foundation - data infrastructure, H3 (PyQt slice OBSOLETE — React/Django/deck.gl shipped instead, specs 041/042/061) |
| `epochs/epoch3/` | Epoch 3: The Game - game features (17 specs, renumbered from epoch2/; 3.9 Balkanization shipped as spec-070) |
| `epochs/epoch4/` | Epoch 4: The Platform - vision only (DuckDB unification + ChromaDB superseded by Postgres 16 + pgvector, spec-037/ADR030) |
| `architecture.yaml` | Technical Stack (Embedded Trinity: Ledger, Topology, Archive) |
| `tooling.yaml` | CI/CD, Testing, Parameter Tuning, Monte Carlo UQ |
| `formulas-spec.yaml` | Formula signatures (historical; actual: 55 public functions across 17 modules in `src/babylon/formulas/`) |
| `observer-layer.yaml` | Observer system, EventTypes, TopologyMonitor |

---

## THEORETICAL REFERENCE (YAML + RST)

| YAML | RST | Purpose |
|------|-----|---------|
| `theory.yaml` | `docs/concepts/theory.rst` | MLM-TW theoretical grounding |
| `carceral-equilibrium.yaml` | `docs/concepts/carceral-equilibrium.rst` | 100-year trajectory, revolution windows |
| `epochs/epoch3/warlord-trajectory.yaml` | `docs/concepts/warlord-trajectory.rst` | Epoch 3 branching endgames |
| `terminal-crisis.yaml` | `docs/concepts/terminal-crisis.rst` | Crisis phases: plantation → death camp |
| `epochs/epoch1/demographics.yaml` | `docs/reference/demographics.rst` | Mass Line: population blocks |
| `tuning-standard.yaml` | `docs/reference/tuning.rst` | 20-Year Entropy Standard |

---

## ARCHIVE (Do Not Read for Coding)

| Directory | Purpose |
|-----------|---------|
| `archive/pre_necropolis/` | Old specs from Phase-era nomenclature (pre-2025 pivot) |
| `archive/pre_necropolis/decisions_monolith.yaml` | Original monolithic ADR file (now split into decisions/) |

---

## Quick Reference by Epoch

### Epoch 1: The Engine - COMPLETE

See `epochs/epoch1-complete.md` for full completion record.

| Slice | Name | Status |
|-------|------|--------|
| 1.1 | Core Types | COMPLETE |
| 1.2 | Economic Flow | COMPLETE |
| 1.3 | Survival Calculus | COMPLETE |
| 1.4 | Consciousness Drift | COMPLETE |
| 1.5 | Synopticon Dashboard | COMPLETE |
| 1.6 | Endgame Resolution | COMPLETE |
| 1.7 | Graph Bridge | COMPLETE |
| 1.8 | Carceral Geography | COMPLETE |

### Epoch 2: The Foundation - IN PROGRESS

See `epochs/epoch2/overview.md` for specifications.

| Slice | Name | Status |
|-------|------|--------|
| 2.1 | 3NF Schema | COMPLETE |
| 2.2 | Census Loaders | COMPLETE |
| 2.3 | Economic Loaders | COMPLETE |
| 2.4 | Circulatory Loaders | COMPLETE |
| 2.5 | H3 Geographic System | COMPLETE (shipped at res-7, not the spec's res-4) |
| 2.6 | PyQt Visualization | OBSOLETE (superseded by React 19 + Django + deck.gl, specs 041/042/061) |
| 2.7 | Schema Integration | PARTIAL |
| 2.8 | LODES & Freight Flows | PARTIAL |
| 2.9 | Ideological Cartography | PARTIAL |

**Epoch 2 Spec Files**: `epochs/epoch2/`
- `data-infrastructure.yaml`, `data-quality.yaml` (2.1-2.4)
- `census-data-loader.yaml`, `fred-api.yaml`, `census-insights.yaml` (detail specs)
- `h3-geographic-system.yaml` (2.5)
- `pyqt-visualization.yaml`, `echarts-patterns.yaml` (2.6)
- `schema-integration.yaml`, `graph-protocol.yaml` (2.7)
- `lodes-freight-flows.yaml` (2.8)
- `ideological-geography.yaml` (2.9)

### Epoch 3: The Game - IN PROGRESS

Substrate ~60-70% built via specs 011-066; slice 3.9 (Balkanization) shipped as spec-070; 3.7 partially covered by spec-039 (state-apparatus AI). See `reports/aidocs-vs-code-audit-2026-05-16.md` Part 3-FULL for the per-slice verdicts and the forward catalog. All vision specs in `epochs/epoch3/`:

| Slice | File | Purpose |
|-------|------|---------|
| 3.1 | demographics-spec.yaml | Demographic Resolution |
| 3.2 | vanguard-economy.yaml | Cadre/Sympathizer/Reputation Resources |
| 3.3 | cohesion-mechanic.yaml | Internal Dynamics, Iron Law of Oligarchy |
| 3.4 | fog-of-war.yaml | Fish in Water Intel |
| 3.5 | gramscian-wire-vision.yaml | Narrative Warfare, Hegemony |
| 3.6 | repression-logic.yaml | State Repression Mechanics |
| 3.7 | state-attention-economy.yaml | State AI Threads |
| 3.8 | kinetic-warfare.yaml | Asymmetric Logistics, System Disruption |
| 3.9 | balkanization-spec.yaml | Territorial Fracture |
| 3.10 | doctrine-tree.yaml | Ideological Tech Tree, Line Struggle |
| 3.11 | strategy-layer.yaml | External Actions, Resource Traps |
| 3.12 | reactionary-subject.yaml | Class Basis of Fascism |

### Epoch 4: The Platform - VISION

See `epochs/epoch4/overview.md` for specifications.

---

## Architecture & Patterns

| File | Purpose |
|------|---------|
| `game-loop-architecture.yaml` | Engine/State separation, Systems, formulas |
| `entities.yaml` | 17 game entity types and relationships |
| `decisions/index.yaml` | Legacy ADR registry used by deterministic sentinels. Query with `mise run adr` |
| `ontology.yaml` | Domain terms (MLM-TW, architecture, game) |
| `patterns.yaml` | How to do things here |
| `anti-patterns.yaml` | What NOT to do |
| `mantras.yaml` | Guiding principles as memorable phrases |
| `pydantic-patterns.yaml` | Pydantic V2 models, validators, serialization |
| `documentation-standards.yaml` | Diataxis, RST format, docstrings |

**ADR Files:** Individual ADRs are in `decisions/` directory (e.g., `decisions/ADR001_embedded_trinity.yaml`)

---

## UI & Visualization

| File | Purpose |
|------|---------|
| `epochs/epoch1/synopticon-spec.yaml` | Epoch 1 Observer system architecture, Lens filter |
| `epochs/epoch1/ui-wireframes.yaml` | ASCII wireframes (Epoch 1 DearPyGui layout) |
| `epochs/epoch1/dpg-patterns.yaml` | DearPyGui patterns (Epoch 1 reference) |
| `epochs/epoch2/pyqt-visualization.yaml` | Epoch 2 PyQt + pydeck visualization |
| `epochs/epoch2/echarts-patterns.yaml` | Apache ECharts chart configuration |

---

## Usage

When starting a session, an AI assistant should:
1. Read `reports/aidocs-vs-code-audit-2026-05-16.md` for the living roadmap (`ai/epochs/overview.md` is the historical vision)
2. Read `ai/state.yaml` for current sprint status and test counts
3. Reference `ai/ontology.yaml` when encountering domain terms
4. Check `ai/patterns.yaml` before implementing new features
5. Run `mise run adr -- search "<topic>"`
6. Run `mise run adr -- show ADRNNN` for one result. Read only its returned source at the root selector

## Maintenance

Update these docs when:
- New domain concepts are introduced
- Architectural decisions are made
- Patterns emerge or change
- Mistakes are made (add to anti-patterns)
