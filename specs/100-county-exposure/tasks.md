# Tasks: County-Exposure Loader (spec-100)

**Feature**: `100-county-exposure` | **Program**: 09 Lane D
**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

Two repos. `WT` = babylon worktree `/home/user/projects/game/babylon/worktrees/d100`
(schema + mise task + spec docs). `BD` = babylon-data repo
`/home/user/projects/game/babylon-data` (branch `100-county-exposure`; the loader).
Commit after each unit in BOTH repos (WT: `mise run commit`; BD: plain git, no hooks).
TDD: write the failing test FIRST, run it, observe RED (`@pytest.mark.red_phase`),
then GREEN. Run tests with `PYTHONPATH=src` (never bare `poetry run`).

## Phase 1: Setup / Foundational (schema)

- [ ] T001 [WT] Add additive ORM class `FactCountyExposureByExternal` to `src/babylon/reference/schema.py` (PK time_id, external_country_id, county_id; weight Float NOT NULL; FKs to dim_time/dim_country/dim_county; index on (time_id, external_country_id); RST docstring citing the phi_distribution consumer).
- [ ] T002 [WT] Add additive ORM class `FactBilateralTradeAnnual` to `src/babylon/reference/schema.py` (PK time_id, country_id; imports/exports/total_trade Numeric(18,2) nullable; FKs; docstring noting it feeds ExternalNode.bilateral_trade_value USD).
- [ ] T003 [WT] Write `tests/unit/reference/test_exposure_schema.py`: create both tables in an in-memory SQLite engine from the ORM metadata, insert + read back a row for each, assert PK/column shape. Run RED (tables not yet importable if T001/T002 incomplete), then GREEN.
- [ ] T004 [WT] `mise run check:quick` (lint+format+typecheck on schema) green; commit unit: `feat(spec-100): additive county-exposure + bilateral-trade reference tables`.

## Phase 2: US1 — county import-exposure map (Priority P1) 🎯 MVP

**Goal**: compute per-(bloc,year) county weights summing to 1.0 and persist via staged swap.
**Independent test**: seed fixture reference DB, dry-run, assert Σweight=1.0 and top counties match seeded import-intensive industries.

- [ ] T005 [BD] Create `src/babylon_data/exposure/__init__.py` (empty-imports guard, qcew precedent: `__all__: list[str] = []`, docstring).
- [ ] T006 [P] [BD] [US1] Write `tests/unit/data/exposure/conftest.py`: in-memory ORM engine seeding helpers — builders for dim_time/dim_country/dim_county/dim_industry/dim_ownership/dim_bea_industry/bridge_naics_bea/fact_bea_io_coefficient/fact_qcew_annual/fact_trade_monthly. Seed a small deterministic scenario (2 blocs, 3 counties, a Noncomparable-imports BEA source, 2-3 covered manufacturing BEA industries, an antichain concordance, ownership rows {1,2,3,5}).
- [ ] T007 [BD] [US1] Write `tests/unit/data/exposure/test_compute.py` RED (`@pytest.mark.red_phase`): assert `compute_exposure(session, year)` returns per-(bloc,county) weights summing to 1.0±1e-9, weights in [0,1], a zero-import-coeff-industry county gets 0, split-NAICS apportioned by 1/split_count, covered-industry set correct. Run, observe RED (module absent).
- [ ] T008 [BD] [US1] Implement `src/babylon_data/exposure/compute.py`: pure formula (data-model.md §Compute). Resolve the Noncomparable-imports BEA id + USE table_type_id by lookup (not literal). Sum QCEW owns {1,2,3,5}. Return frozen dataclass(es): the exposure map + coverage/reconciliation aggregates. GREEN T007.
- [ ] T009 [BD] [US1] Write `tests/unit/data/exposure/test_writer.py` RED: staged `__new` tables created from ORM DDL; `write_year` inserts sorted rows; `swap_staging` promotes + keeps `__pre_100`; `rollback_from_backup`/`drop-backup`; `logical_table_hash` identical across two builds of the same fixture; absent-table sentinel. Observe RED.
- [ ] T010 [BD] [US1] Implement `src/babylon_data/exposure/writer.py` mirroring spec-086 `qcew/writer.py` (build_dim_maps, staging tables, per-year transactional load, atomic swap w/ backup, rollback, drop_backup, logical_table_hash). Handle first-ever build (no canonical to back up). GREEN T009.
- [ ] T011 [BD] [US1] Write `tests/unit/data/exposure/test_validation.py` RED: `sum_to_one_gate` fails a tampered weight vector; `reconcile_year` computes Σraw vs Σcovered coeff and the residual %, `within_2pct` boolean. Observe RED.
- [ ] T012 [BD] [US1] Implement `src/babylon_data/exposure/validation.py` (sum-to-one gate + reconciliation, named constants for ±2% band and 1e-9 tolerance with provenance comments). GREEN T011.
- [ ] T013 [BD] [US1] Run `PYTHONPATH=src uv run pytest tests/unit/data/exposure -q` (compute+writer+validation green); commit BD unit: `feat(exposure): US1 compute + staged writer + validation (spec-100)`.

## Phase 3: US2 — bloc-year bilateral trade (Priority P2)

**Goal**: aggregate fact_trade_monthly → fact_bilateral_trade_annual (exact sums).

- [ ] T014 [BD] [US2] Neutralize `src/babylon_data/trade/__init__.py` (remove stale `babylon.data.*` imports; `__all__: list[str] = []` + docstring noting legacy modules retained as provenance — qcew precedent). Verify `import babylon_data.trade` succeeds.
- [ ] T015 [BD] [US2] Write `tests/unit/data/exposure/test_bilateral.py` RED: seed monthly rows for 2 blocs × 2 years (some months absent); assert annual = exact sum per bloc-year for imports/exports/total; absent month contributes nothing. Observe RED.
- [ ] T016 [BD] [US2] Implement `src/babylon_data/trade/bilateral.py`: `aggregate_bilateral_annual(session, years)` over `is_region=1` blocs (current `babylon.reference` imports). GREEN T015.
- [ ] T017 [BD] [US2] Run trade tests green; commit BD unit: `feat(trade): US2 bloc-year bilateral aggregation + neutralize __init__ (spec-100)`.

## Phase 4: US3 — audit contract (Priority P2)

**Goal**: schema-validated JSON + Markdown audit per run.

- [ ] T018 [BD] [US3] Write `tests/unit/data/exposure/test_audit.py` RED: build a `CountyExposureAuditReport` from fixture aggregates; assert `to_json()` validates against `specs/100-county-exposure/contracts/exposure_audit.schema.json`; Markdown render non-empty. Observe RED.
- [ ] T019 [BD] [US3] Implement `src/babylon_data/exposure/audit.py`: frozen Pydantic models mirroring the JSON Schema (RunMetadata, YearEntry, Reconciliation, ConcordanceCoverage, TradeSummary, Gates), `git_ref`, `sha256_of_file`, `write_to_disk` (jsonschema-validate against the contract when reachable), `_render_markdown`. GREEN T018.
- [ ] T020 [BD] [US3] Run audit tests green; commit BD unit: `feat(exposure): US3 audit contract + JSON/MD emit (spec-100)`.

## Phase 5: CLI + mise task

- [ ] T021 [BD] Implement `src/babylon_data/exposure/__main__.py`: argparse CLI (`--dry-run/--apply/--rollback-from-backup/--drop-backup/--years/--db/--report-dir`), preflight (source tables non-empty), per-year process → validate → (apply) staged load → gate → swap → emit audit. Exit codes 0/1/2/130. Mirror `qcew/__main__.py`.
- [ ] T022 [WT] Add `[tasks."data:exposure"]` to `.mise.toml` (usage flags mirroring data:qcew; `uv run python -m babylon_data.exposure ...`). `mise trust` if needed.
- [ ] T023 [BD] Write `tests/unit/data/exposure/test_cli.py`: `main(["--dry-run","--years","<fixtureyear>","--db",<tmp>])` returns 0 and writes an audit artifact; a tampered fixture returns exit 1 with canonical untouched. (Uses a temp seeded DB.) Green.
- [ ] T024 [BD] commit BD unit: `feat(exposure): US? CLI entry point (spec-100)`; T024b [WT] commit `feat(spec-100): data:exposure mise task`.

## Phase 6: Integration (IV Michigan gate) + apply

- [ ] T025 [BD] Write `tests/integration/data/exposure/test_exposure_real_db.py` (`@pytest.mark.integration`): copy the real reference DB (VACUUM INTO a temp) OR open read-only + compute in memory for Michigan tri-county (Wayne 26163, Oakland 26125, Macomb 26099) for 2024; assert per-bloc weights over those counties sum to 1.0 when restricted, Wayne's manufacturing raw-exposure > 0, reconciliation within ±2%. Gate skips gracefully if the real DB is absent.
- [ ] T026 Run `mise run data:exposure -- --dry-run --years 2024` against the real DB; capture audit; verify reconciliation within_2pct=true, coverage reported, hashes present.
- [ ] T027 Run `mise run data:exposure -- --apply` (full 2010–2024) against the real DB; verify exit 0, both tables populated, per-(bloc,year) sum=1.0, `logical_table_hash` reproduces on a second dry-run. Commit any audit artifact per precedent.

## Phase 7: Close-out

- [ ] T028 [WT] Update `project/01-state-of-the-world.md` (spec-100 DONE), `project/09-program-full-game.md` §2 spec-100 status, `ai-docs/state.yaml`.
- [ ] T029 Final commits in both repos; write report to `.superpowers/sdd/reports/100.md`.

## Dependencies

- T001–T004 (schema) block all BD tasks (loader imports the ORM).
- US1 (T005–T013) is the MVP; US2/US3 independent of each other, both need US1's writer/compute types only lightly (audit consumes aggregates).
- CLI (T021) needs compute+writer+validation+audit+bilateral.
- Integration/apply (T025–T027) need CLI + real DB + mise task.

## Parallel opportunities

- T006 (fixtures) ∥ T001/T002 (schema) once ORM class names are fixed.
- After US1: T014–T016 (trade) ∥ T018–T019 (audit).

## MVP scope

US1 (Phase 2) alone delivers the gating deliverable: the computed
`county_exposure_by_external` map persisted deterministically. US2/US3 complete
the audit + trade surface for spec-101.
