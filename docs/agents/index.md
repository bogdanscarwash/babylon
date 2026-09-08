# Babylon Wiki Index

Content-oriented catalog of all agent-facing documentation. Updated after every wiki ingest.

## Architecture & Code

- [Architecture Overview](architecture.md) — canonical Rust, persistence, and observer boundary
- [Coding Standards](coding-standards.md) — Pydantic, TDD, commits, docstrings, imports

## Operations

- [Testing](testing.md) — proportionate native and data-tool validation
- [Commands](commands.md) — mise tasks reference
- [Gotchas](gotchas.md) — Common pitfalls and debugging lessons

## Governance

- [Governance](governance.md) — Git workflow, session continuity, ai maintenance

## Architecture Decision Records

- [ADR Index](adrs/README.md) — refactoring sequence proposed against the knowledge graph
  - [ADR-001](adrs/ADR-001-mechanical-file-splits.md) — Mechanical splits (defines.py, enums.py, OODA dedup)
  - [ADR-002](adrs/ADR-002-protocol-kit-and-source-registry.md) — protocol_kit + SourceRegistry
  - [ADR-003](adrs/ADR-003-system-abc.md) — Lift `System` Protocol to ABC
  - [ADR-004](adrs/ADR-004-discriminated-event-union.md) — Discriminated `TickEvent` union
  - [ADR-005](adrs/ADR-005-god-class-decomposition.md) — Decompose `postgres_runtime.py` + `simulation.py`
  - [ADR-006](adrs/ADR-006-cleanup-batch.md) — Scenario ABC + remaining splits + orphan schemas

## Specifications

- Plans: `.specify/plans/`
- Specs: `.specify/specs/`
- Constitution: `CONSTITUTION.md`
- Data Catalog: `data-catalog.yaml`

## AI State

- Decisions: `ai/decisions.yaml`
- Roadmap: `ai/roadmap.md`
- State: `ai/state.yaml`

## Agent Skills

- `.opencode/skills/specification-discover/` — Problem decomposition and scope control
- `.opencode/skills/specification-plan/` — Design artifacts and implementation planning
- `.opencode/skills/specification-specify/` — API contracts and interface definitions
- `.opencode/skills/specification-build/` — TDD implementation and code generation
- `.opencode/skills/specification-validate/` — Tests, checklists, and acceptance criteria
- `.opencode/skills/specification-govern/` — Constitutional compliance and amendment review
- `.opencode/skills/wiki-maintain/` — Wiki ingest, query, and lint operations

## Changelog

- [Wiki Log](log.md) — chronological ingest history
