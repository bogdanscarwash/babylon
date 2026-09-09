# Phase 0-D — The Spatial Data Charter (Standard first-moves item 4)

**Status:** PRODUCTS BUILT (PR #401, 2026-07-30) — all three artifacts registered in `data-artifacts.yaml`; consumer wiring design-gated on the P27 multi-res shape.
**Authority:** ADR176 ruling 18 (*DATA FIRST; Phase 0-D BLOCKS multi-res engine
work; the land/water MASK IS IN SCOPE; no declared-fabrication interim*);
the Game Design Standard §6/§10; the design-inputs dossier §§3.2/819/830.

## The defect this charter exists to fix

- `bridge_county_h3.coverage_pct` is **uniformly 100** across all 45,572
  res-7 cells (verified live, 2026-07-30) — the fabrication the dossier
  named: ~40% of measured Michigan res-7 cells are over open water
  (`dim_county_geometry` totals 250,486 km² for the 83 bridge counties;
  Michigan's land is ~146,000 km² — the county polygons extend into the
  Great Lakes), and nothing in the pipeline knows.
- No sub-county share key exists: the hex hydrator's "uniform within
  county" allocation is declared fabrication of structure (S-12), and a
  uniformly-refined cell has zero closure defect by construction — the
  multi-res trigger would measure its own assumption.

## The three build products (ADR098 shape: sha-pinned, CI consumes, never builds)

| # | Artifact | Grain | Source (declared acquisition) | Retires |
|---|---|---|---|---|
| 1 | `h3_res7_land_mask.parquet` — `h3_index, county_fips, land_fraction` | res-7 cell | TIGER/Line 2023 AREAWATER (the 83 bridge counties) intersected with the res-7 tiling | the inert `coverage_pct` (recomputed as real land fraction) |
| 2 | `h3_res7_population.parquet` — `h3_index, population` | res-7 cell | Census 2020 P.1 at block-group grain + TIGER block-group geometries, areal-apportioned through the mask | the "uniform within county" population smear |
| 3 | `h3_res7_workplace.parquet` — `h3_index, jobs` | res-7 cell | LODES 8 WAC at block grain (`w_geocode`), block→cell assignment through the mask | the missing workplace-density share key (the dossier's finding: `fact_lodes_commuter_flow` is county→county, not per-block) |

## Acquisition discipline

The babylon-data trove holds county-level TIGER only (verified). The
missing inputs (AREAWATER + block groups + blocks + LODES WAC for the
bridge counties) enter through a **fetch manifest** — pinned URLs +
sha256 per file, committed beside the builder — never an ad-hoc download.
The build runs LOCALLY against the trove (CI never touches the drive —
standing rule); the artifacts commit as sources with their shas in
`data-artifacts.yaml`, exactly the reference-DB pipeline's discipline
(`tools/build_reference_db.py` precedent; `docs/how-to/reference-data-pipeline.rst`).

## Order of work

1. The fetch manifest + `tools/build_phase0d_artifacts.py` scaffold with
   schema-pinned outputs (tests red until built).
2. Product 1 (the mask) — build, pin, register; recompute/retire
   `coverage_pct` consumers.
3. Products 2+3 (same acquisition family) — build, pin, register.
4. The two-spatial-grain reconciliation (the hydrator reads the mask +
   share keys; the "uniform within county" comment retires; the
   ContentDigest question of share-keys-as-content goes to the P27 Phase-4
   design per the dossier's finding 3).

## What this charter does NOT do

No multi-res engine work (ruling 18 blocks it until these exist); no
grain-register or aggregate_constraint riders (ruling 19 — those ride the
cutover ceremony); no fabricated interim values anywhere.
