# Tasks — spec-107 σ-gradient (dependency-ordered)

Legend: [x] done · [ ] todo · [~] deliberately not done this unit (blocked or
out of scope, disposition stated). Each numbered task = one commit unit in
the shipped record (this authoring pass); items marked `[ ]`/`[~]` are
handed to a future unit, not this one.

- [x] **T1 — Research.** Verify current LOADED/STAGED/AMPUTATED status of
  every σ data ingredient against `reference/schema.py`,
  `data-catalog.yaml`, `data-artifacts.yaml`, `ADR075`, `ADR076`, the
  amputation reports, and the two Ricci-shaped constructs. `research.md`
  R1–R8.
- [x] **T2 — RED: statistics.** `tests/unit/domain/economics/sigma/test_statistics.py`
  written first (mean/std/z-score contract + degenerate-input error
  contract); watched fail on import.
- [x] **T3 — GREEN: statistics.** `src/babylon/domain/economics/sigma/statistics.py`
  (`compute_distribution_stats`, `z_score`) + `types.py`'s `DistributionStats`.
- [x] **T4 — RED: components.** `test_components.py` written first (OCC, K/L,
  ℓ dot-product + error contracts).
- [x] **T5 — GREEN: components.** `components.py`.
- [x] **T6 — RED: composite.** `test_composite.py` written first
  (standardize + compose, including the `ComponentWeights` sum-validation
  contract).
- [x] **T7 — GREEN: composite.** `composite.py` + `types.py`'s
  `SigmaComponents`/`ComponentWeights`/`ComponentDistributionStats`/
  `StandardizedComponents`.
- [x] **T8 — RED: anchor.** `test_anchor.py` written first (Owner-Ruling-1
  "US anchors above world mean" behavior).
- [x] **T9 — GREEN: anchor.** `anchor.py`.
- [x] **T10 — RED: wage alignment.** `test_wage_alignment.py` written first
  (OLS fit + target + deviation + the negative-target Loud-Failure
  contract).
- [x] **T11 — GREEN: wage alignment.** `wage_alignment.py` + `types.py`'s
  `WageTargetModel`.
- [x] **T12 — RED-phase contract tests.** `test_contracts_red_phase.py`:
  three tests pinning current absence (σ-index artifact file, `"spectrum"`
  in `_OPPOSITION_FIELD_NAMES`, `SpectrumDefines` on `GameDefines`) —
  written to PASS today, expected to eventually fail as a build signal.
- [x] **T13 — Package surface.** `__init__.py` re-exporting the full public
  API with `__all__`.
- [x] **T14 — Verify loop.** `mise run test:q -- tests/unit/domain/economics/sigma`
  (38 passed) → `uv run ruff check src/babylon/domain/economics/sigma
  tests/unit/domain/economics/sigma` (clean) → `uv run mypy
  src/babylon/domain/economics/sigma` (clean, strict mode).
- [x] **T15 — Speckit artifacts.** `spec.md`, `plan.md`, `tasks.md` (this
  file), `research.md`.

## Handed to future units (not this unit's write surface or scope)

- [ ] **Fixed-assets loader (FAAt3.1ESI).** ORM class + migration in
  `reference/schema.py`, loader in the babylon-data repo (standing owner
  ruling on loader home). Blocks a real (non-proxy) `capital_stock` input
  to this package's `components.py` functions. Owner: whoever picks up
  program-10 §9's original next-action; unaffected by this authoring pass.
- [ ] **σ-index artifact generation.** `tools/make_sigma_index_artifact.py`
  (reads the reference DB once, at authoring time, never at runtime/CI) +
  the `src/babylon/data/reference/sigma_index.parquet` file + its
  `data-artifacts.yaml` registration (ADR121-pattern hash-stamp). **BLOCKED
  in this worktree**: the `babylon-data` drive the ingest scripts read from
  is absent (dangling symlinks). Pinned by
  `test_sigma_index_artifact_not_yet_generated`.
- [ ] **Hydration adapter.** A future `sigma_data_sources.py`-shaped module
  (the `county_exposure.py` pattern: pure sqlite reads, no engine coupling)
  that reads `fact_qcew_annual` / `fact_bea_io_coefficient` /
  `sigma_index.parquet` and calls into this package's pure functions. Not
  written here (Decision D5: this package does no I/O).
- [ ] **`SpectrumDefines` GameDefines category.** A new `config/defines/spectrum.py`
  sub-model (following the `_assembler.py` convention, research.md R6)
  holding: `weight_occ`, `weight_capital_intensity`, `weight_labor_content`
  (this package's `ComponentWeights`, canonical values TBD — Director ruling
  #1), and the wage-gravitation rate (coupling 2's per-tick pull toward
  ŵ(σ), not built anywhere in this package). Requires regenerating
  `defines.yaml` — both are outside this unit's write surface. Pinned by
  `test_spectrum_defines_category_not_yet_added`.
- [ ] **Engine wiring (U5, post-P25).** The four consumption seams in
  spec.md § Consumption seams: the `"spectrum"` field entry
  (`contradiction_field.py`), the `APEX_LABOR ⊣ BASE_LABOR` opposition
  (`dialectics/instances/catalog.py`), the per-edge transfer lever
  (`economic.py`, activating the already-registered-but-uncalled
  `formulas/unequal_exchange.py` functions), and the consciousness coupling
  (activating `calculate_labor_aristocracy_ratio`). Explicitly out of scope
  per Program 26 §4 (U5 is gated post-P25) and forbidden by this unit's own
  hard constraints (no `engine/` change).
- [ ] **Director rulings (spec.md, recorded, not resolved here):**
  1. Confirm or replace the z-score-standardization + linear-weighted-sum
     composite-σ method (Decision D1); supply canonical `ComponentWeights`
     values.
  2. (Folds into #1) Given the Hickel series is national-aggregate-only
     (Decision D2), what is the actual per-bloc/per-node sample that grounds
     `world_stats`?
  3. Whether to re-ingest `babylon_ricci_final.csv`'s Andrea Ricci
     core/periphery/semi-periphery UE data as a live reference-DB table
     (undoing or paralleling the 2026-07-17 amputation), since it is the
     closest thing on disk to program-10's originally-intended second
     anchor input (Decision D3).
