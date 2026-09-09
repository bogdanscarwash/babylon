# spec-107 — The Spectrum of Unequal Exchange (σ-gradient)

**Program**: 10 (theory, ratified 2026-07-08, `project/programs/
10-spectrum-of-unequal-exchange.md`) — authored under Program 26 (International
Trade) Unit U1, `project/programs/26-international-trade.md`. **Depends on**:
nothing at the code level (off-pipeline, pure math). **Consumed by** (future,
not this unit): U5 — the post-P25 international-layer engine train.
**Status**: authored (this document); domain math + red-phase contract tests
shipped alongside it; engine wiring is explicitly out of scope.

## Why

Program 10 was ratified as theory nine months before Program 26 reopened
trade, but `specs/107-*` was never written — the ratification's own
next-actions checklist named that as "next session, first item" and it sat
undone. Program 26's audit (ADR160) confirmed the four dormant
`formulas/unequal_exchange.py` functions still have zero registry callers and
`specs/107-sigma-gradient/` still didn't exist. This spec is that missing
document: it pins the σ-gradient's math and data contract precisely enough
that a future engine-wiring unit (U5) has something concrete to consume,
without touching any engine file itself (Program 26 §3's non-overlap covenant
with the P25 lane forbids that here regardless).

Percy's formulation (program-10 §1, verbatim intent): labor transfers from
colony to metropole align along a **scale**. At the apex sits high organic
composition of capital, high capital intensity, specialized high-tech labor;
at the base sits high variable capital, low capital intensity. This is the
Amin/Emmanuel/Wallerstein unequal-exchange thesis; the game systematizes it as
a **continuous gradient**, not a core/periphery binary.

## What ships (functional requirements)

- **FR-107-1** — A pure-math package,
  `src/babylon/domain/economics/sigma/`, computing the three raw σ
  ingredients from explicit inputs: organic composition of capital
  (`compute_organic_composition`, OCC = K/v), capital intensity
  (`compute_capital_intensity`, K/L), and vertically-integrated labor content
  (`compute_vertically_integrated_labor_content`, ℓ — the Pasinetti
  construction: one row of the BEA TOTAL_REQ Leontief inverse dotted against
  QCEW labor coefficients). No reference-DB or sqlite I/O happens in this
  package — every function takes already-hydrated mappings/floats (Decision
  D5).
- **FR-107-2** — A documented, explicit composition step
  (`standardize_components` + `compose_sigma`) combining the three
  ingredients into one raw composite σ per node-year, using z-score
  standardization against an explicit `ComponentDistributionStats` and
  explicit, caller-supplied `ComponentWeights` (no hardcoded weights
  anywhere — Decision D1, a Director-ruling item).
- **FR-107-3** — A world-scale anchoring step (`anchor_to_world_scale`)
  standardizing the composite σ against a world-scale reference distribution,
  realizing Owner Ruling 1 ("ONE global axis... US hexes occupy the upper
  band... external boundary nodes sit low on it") as a single reusable
  z-score primitive (`babylon.domain.economics.sigma.statistics.z_score`).
- **FR-107-4** — Wage-gravitation math for coupling 2 (`fit_linear_wage_target`,
  `compute_wage_target`, `compute_wage_deviation`): an OLS fit of
  ŵ(σ) = slope·σ + intercept and the deviation δ = w − ŵ(σ). Owner Ruling 2
  ("wages align, don't define") is honored by construction: wage data is
  never an *input* to σ itself, only to this separate alignment step.
  Monotonicity (slope > 0) is the acceptance criterion Program 10 states
  (program §7), not a constraint this code enforces — see Decision D6.
- **FR-107-5** — A data contract (§ Data contract, below) documenting exactly
  what is currently LOADED vs. STAGED vs. AMPUTATED for σ's inputs, current
  as of this authoring pass (2026-07-27) — correcting several claims in the
  2026-07-08 program doc that have since drifted (Decisions D2, D3).
  Generating the precomputed σ-index artifact itself is **environment-blocked**
  in this worktree (the `babylon-data` drive the loaders read from is absent)
  and is out of scope for this unit — see tasks.md.
- **FR-107-6** — Red-phase contract tests
  (`tests/unit/domain/economics/sigma/test_contracts_red_phase.py`) pinning
  three deliberately-unbuilt items so their eventual construction is a loud,
  caught event rather than a silent drift: the σ-index artifact's absence,
  the `"spectrum"` field's absence from `engine/systems/contradiction_field.py`'s
  `_OPPOSITION_FIELD_NAMES`, and the absence of a `SpectrumDefines` category
  on `GameDefines`.
- **FR-107-7** — Consumption seams declared (not inserted) for U5: the field
  entry point (System 19's `_OPPOSITION_FIELD_NAMES` + System 20's automatic
  gradient/Laplacian machinery), the opposition entry point
  (`APEX_LABOR ⊣ BASE_LABOR` in `dialectics/instances/catalog.py`), the
  transfer-lever entry point (`economic.py`'s national-flat
  `extraction_efficiency`, replaceable per-edge via the already-dormant
  `formulas/unequal_exchange.py` functions), and the consciousness-coupling
  entry point (the already-dormant `calculate_labor_aristocracy_ratio`,
  `formulas/fundamental_theorem.py`) — enumerated in § Consumption seams,
  none touched by this unit.

## Non-goals

- Any engine, system, or tick-pipeline change (Program 26 §3's non-overlap
  covenant with P25 forbids `engine/systems/*` in this unit regardless of
  content).
- Position mobility (dσ/dt from capital deepening/disinvestment) — program-10
  §8 item 1, explicitly deferred past "slice 1."
- Generating the σ-index data artifact — environment-blocked (§ Data
  contract); the schema is specified here, the file is not produced.
- Settling the Φ-attribution model — that is Program 26 Unit U4's job,
  gated on a Director ruling, and orthogonal to σ (σ is production
  structure; Φ-attribution is how imperial rent is split across blocs).
- BLS EP I-O cross-check, BEA MAKE/SUPPLY finer industries, Wallerstein
  acquisition, spectrum-conditioned edge-mode transitions, σ-on-the-map — all
  program-10 §8 deferred items, unchanged by this authoring pass.

## Key decisions (recorded)

> **D1–D3 RULED/DELEGATED 2026-07-27 — see
> `ai/decisions/ADR165_p26_director_rulings_trade_slate.yaml`.** D1: delegated; agent
> decision = confirm z-score standardization + linear weighted sum, canonical weights
> (1/3, 1/3, 1/3) homed in `defines.yaml`. D2/D3: re-ingest `babylon_ricci_final.csv` as a
> reference-DB table (declared partial undo of the 2026-07-17 amputation) to ground
> `world_stats`. σ-composition (Option C) is the RULED Φ-attribution model — this spec's
> U5 consumption seams are now on the critical path.

- **D1 — The composite-σ combination formula is a Director-ruling item, not
  settled theory.** Program 10 §3 specifies the three ingredients precisely
  (OCC, K/L, ℓ) but says only "Composite σ per BEA-industry×year" — no
  weighting scheme, no normalization method is stated. This spec's domain
  math implements a documented DEFAULT (z-score standardize each raw
  component against its own cross-sectional distribution, then combine with
  explicit caller-supplied weights) so the math is testable and the gap does
  not block authoring — but the *choice* of z-score (vs. min-max, vs. rank,
  vs. a Cobb-Douglas-style geometric form) and the canonical weight values
  are flagged here as requiring Director confirmation before U5 wires any of
  this into the engine. **Director ruling required (#1)**: confirm or
  replace the z-score-standardization + linear-weighted-sum composition
  method, and supply canonical component weights (or a rule for deriving
  them).
- **D2 — The Hickel ERDI series is a single national aggregate; per-bloc
  resolution does not exist.** `fact_hickel_erdi_annual` (LOADED,
  `reference/schema.py:1932`) carries `scale_type ∈ {Extensive, Intensive,
  Intensive_China_Inflection}` — a national time series, not per-partner
  (verified identically by spec-101 R2). Program-10's "normalize on the world
  scale using... FactHickelERDIAnnual... and FactHickelDrain" (§3) already
  anticipated a *separate* per-partner drain table; no such table exists
  today (`FactHickelDrain` was never a schema.py class distinct from
  `FactHickelERDIAnnual`; the one that overlapped in name,
  `fact_hickel_drain`, was **amputated** per ADR075 ruling 1, A14 — the class
  docstring at `reference/schema.py:1923` records this explicitly). This
  spec's `anchor_to_world_scale` therefore takes an explicit
  `DistributionStats` computed from whatever world-scale sample is available
  at hydration time — it does not assume a specific table. **Director ruling
  required (#2, informational, folds into #1)**: given the Hickel series is
  national-only, what IS the per-bloc/per-node raw-composite sample that
  grounds `world_stats`? See D3 for the leading candidate.
- **D3 — "FactRicciUnequalExchange" does not exist as program-10 described
  it; a genuine, better-fitting substitute exists in-repo.** Program-10 §3
  cites `FactRicciUnequalExchange` as a loaded ORM class supplying a second
  anchor input. It never existed under that name. What exists:
  (a) `fact_ricci_unequal_exchange` — an actual sqlite table (29 rows) that
  **was amputated** 2026-07-17 (`reports/amputation_demotion_20260717_011045.md`;
  demoted to CSV-only per ADR076 R2 — "the in-repo CSV is already
  canonical"); its data survives ONLY as the checked-in
  `src/babylon/data/reference/babylon_ricci_final.csv` (51 rows, years
  {1995, 2000, 2007, 2009}, columns `region_name, region_type
  (CORE/SEMI_PERIPHERY/PERIPHERY), flow_direction (INFLOW/OUTFLOW),
  transfer_type (GVC/TOTAL), value_usd_billions, value_pct_gdp,
  signed_value, gvc_share_of_total`) — this is genuine Andrea Ricci
  global-value-chain unequal-exchange data (Ricci, 2019, "Unequal Exchange in
  the Age of Global Value Chains"), region-classified almost exactly onto
  Program 26's 8 engine nodes (`China`, `India`, `Russia and CSI`,
  `Southeast Asia`, `Sub-Saharan Africa`, `North America`, `Western Europe`
  all appear as `region_name` values); (b) a *separate*, confusingly
  same-named runtime table `immutable_reference_ricci_unequal`
  (`persistence/migrations/0010_immutable_reference_tables.sql:71`) that is
  actually populated from **Census `fact_trade_monthly` bilateral trade**
  (`persistence/sqlite_hydrator.py::_copy_ricci_unequal`) — unrelated data
  under a legacy label, per spec-101 R4's finding ("`bilateral_trade_value`
  comes from `_copy_ricci_unequal`... NOT the program text's assumed Ricci
  UE series"). **Director ruling required (#3)**: the CSV in (a) is a real,
  better-fitting core/periphery/semi-periphery UE dataset than anything
  currently wired for trade — should it be re-ingested as its own
  sqlite/reference-DB table (undoing the 2026-07-17 amputation, or adding a
  parallel one) to ground `world_stats`, given it is the closest thing on
  disk to what program-10 actually asked for? This is squarely
  theory/data-provenance content under IX.5, not an agent call.
- **D4 — The FAAt3.1ESI fixed-assets loader is still STAGED, not LOADED.**
  Re-verified 2026-07-27 (was already true 2026-07-08): no
  `reference/schema.py` ORM class exists for BEA Fixed Assets net stock by
  industry. `compute_organic_composition` and `compute_capital_intensity`
  both need `capital_stock` (K) as an input — until the loader lands (owner
  ruling on loader home: the babylon-data repo, per program-10 §3 note),
  callers of this package's component functions must source K from the
  interim flow proxy program-10 §3 names (`intermediate_inputs / wage_bill`
  from `fact_bea_national_industry`) or from
  `babylon.domain.economics.capital_stock.CapitalStockCalculator` (perpetual-
  inventory K, already hydrated per Phase-5.2's `TensorRegistry` wiring) —
  either way, this is a hydration-adapter decision for the (not-yet-written)
  U5/index-build unit, not this spec's pure-math package.
- **D5 — This package does no I/O, by design.** Every prior "domain math"
  module surveyed (`county_exposure.py`, `capital_stock.py`,
  `derived_rates.py`) either reads sqlite directly or is fed already-hydrated
  values by a caller that does. Because generating the σ-index artifact is
  environment-blocked in this worktree (§ Data contract), this package takes
  the second shape unconditionally: every function's inputs are explicit
  floats/mappings/frozen models, never a `sqlite_path` or connection. The
  hydration adapter (reading `fact_qcew_annual` / `fact_bea_io_coefficient` /
  the σ-index artifact and calling into this package) is declared as a
  follow-up task, not built here.
- **D6 — Wage-target monotonicity is an empirical claim, not a type
  constraint.** `fit_linear_wage_target` returns whatever slope the OLS fit
  produces, including a non-positive one. Program 10 §7's acceptance
  criterion — "σ rank-correlates strongly and positively with QCEW average
  pay... if this fails, the theory's empirical leg fails loudly, which is the
  point" — is a claim about *real data*, evaluable only once the hydration
  adapter (D5) exists; encoding it as a hard constraint in the fit function
  would silently mask exactly the failure mode program-10 wants surfaced.

## Data contract

Current as of this authoring pass (2026-07-27), verified against
`src/babylon/reference/schema.py`, `data-catalog.yaml`, and
`data-artifacts.yaml`. Where this table disagrees with program-10 §4
(written 2026-07-08), program-10 is stale and this table is the correction —
see Decisions D2–D4 for detail.

| Ingredient | Source | Status |
|---|---|---|
| Wages by industry×county | `fact_qcew_annual` (`total_wages_usd`, `avg_annual_pay_usd`, `employment`) | **LOADED** |
| I-O linkages + Leontief inverse (TOTAL_REQ) | `fact_bea_io_coefficient` (107 `dim_bea_industry` industries, 2010–2024) | **LOADED** |
| Value-added / gross output / intermediate inputs | `fact_bea_national_industry` | **LOADED** |
| NAICS↔BEA mapping | `bridge_naics_bea` | **LOADED** |
| Capital stock by industry (K, the "c" in c/v) | BEA Fixed Assets `FAAt3.1ESI` | **STAGED** — no ORM class in `reference/schema.py` (re-verified 2026-07-27; unchanged since 2026-07-08). Interim proxy: `intermediate_inputs_millions / wage_bill` from `fact_bea_national_industry`, or `CapitalStockCalculator`'s perpetual-inventory K. |
| Commuter flows (residence shift) | `fact_lodes_commuter_flow` | **LOADED** |
| World anchor, series 1 (national aggregate drain) | `fact_hickel_erdi_annual` (`erdi`, `annual_drain_usd_billions`, `scale_type`) | **LOADED, but national-aggregate only** — no per-bloc resolution exists (D2). |
| World anchor, series 2 (core/periphery UE, region-resolved) | `src/babylon/data/reference/babylon_ricci_final.csv` (in-repo, 51 rows) | **CHECKED-IN CSV ONLY** — the backing sqlite table `fact_ricci_unequal_exchange` was amputated 2026-07-17; not currently queryable from the reference DB (D3). |
| BLS EP I-O (jobs per $M final demand) | bls.gov/emp | **MISSING, not needed** — ℓ is derived as TOTAL_REQ × QCEW labor coefficients (program-10 §5, ruling 5). |

### The σ-index artifact (schema, not yet generated)

Following the ADR121 checked-in-artifact pattern (hash-stamped, in-repo,
generated once at authoring time by a script that reads the reference DB —
never at test/CI/runtime, honoring the CI-no-drive rule):

- **Declared path**: `src/babylon/data/reference/sigma_index.parquet`
  (Tier-1 in-repo convention, same home as `babylon_hickel_final.csv` /
  `babylon_ricci_final.csv`).
- **Declared generator**: `tools/make_sigma_index_artifact.py` (not yet
  written — tasks.md).
- **Declared schema** (one row per BEA-industry×year): `bea_industry_id: int`,
  `year: int`, `organic_composition: float`, `capital_intensity: float`,
  `labor_content: float`, `raw_composite_sigma: float` (this package's
  `compose_sigma`, with the eventual Director-ruled weights baked in at
  generation time), `world_anchored_sigma: float | None` (null until a
  world-scale sample exists per D2/D3).
- **Hash-stamp contract**: a `sha256` + row count recorded alongside the
  file, following `data-artifacts.yaml`'s existing `product:`/per-entry
  convention (this spec does not itself edit `data-artifacts.yaml` — that
  registration is a task for whoever writes the generator, since it is
  outside this unit's write surface).
- **Generation status**: **ENVIRONMENT-BLOCKED.** The `babylon-data` drive
  (`/media/user/data/babylon-data/`) this repo's `tools/ingest/*` scripts
  read from is absent on this machine (dangling symlinks, a pre-existing
  condition unrelated to this unit). No artifact is fabricated in its place
  (Constitution III.8) — `tests/unit/domain/economics/sigma/
  test_contracts_red_phase.py::test_sigma_index_artifact_not_yet_generated`
  pins this absence as a loud, checked fact rather than a silent gap.

## Consumption seams (declared, not inserted)

Scout-verified locations (program-10 §5) that U5 will touch — listed here so
this spec is a concrete handoff, not prose. **None of these files are
modified by this unit** (Program 26 §3 non-overlap covenant; also this
spec's own write surface is `specs/107-sigma-gradient/**`,
`src/babylon/domain/economics/sigma/**`,
`tests/unit/domain/economics/sigma/**` only):

| Seam | Location | What U5 would do |
|---|---|---|
| Field entry | `engine/systems/contradiction_field.py:47` `_OPPOSITION_FIELD_NAMES` | Add `"spectrum"`; σ sourced from a future node-attribute hydration, not computed in-tick. |
| Free machinery | `engine/systems/field_derivative.py` `_discover_field_names`, principal-field ranking | Automatic once the field is named — no new code needed there. |
| Opposition | `domain/dialectics/instances/catalog.py` `build_default_registry` | New `APEX_LABOR ⊣ BASE_LABOR` `BoundOpposition`, GapMeasure = population-weighted value-share-vs-labor-share asymmetry along σ. |
| Transfer lever | `engine/systems/economic.py` (national-flat `extraction_efficiency`) | Per-edge ε via `formulas/unequal_exchange.py::calculate_exchange_ratio` / `calculate_value_transfer` (already registered in `formula_registry.py`, zero callers today — this package does not duplicate them). |
| Consciousness coupling | `formulas/fundamental_theorem.py::calculate_labor_aristocracy_ratio` (already exists, zero callers today) | Evaluated per node, keyed to σ, driving per-node consciousness drift. |
| Wage gravitation | `engine/systems/economic.py` wages phase; `reserve_army.py` pressure | Composes with this spec's `compute_wage_target` / `compute_wage_deviation` (FR-107-4) at a defines-tuned rate. |
| Round-trip safety | `models/world_state.py` `SOCIAL_CLASS_COMPUTED_FIELDS` | Whichever unit adds σ as a standalone node attribute must add it to this frozenset in the same commit (C.1 gate) — not this unit's concern since no attribute is added here. |

## Gate (this unit)

- `mise run test:q -- tests/unit/domain/economics/sigma` — green (38 tests:
  35 real math tests + 3 red-phase contract tests, all currently passing —
  the red-phase tests pin absence, not failure).
- `uv run ruff check src/babylon/domain/economics/sigma
  tests/unit/domain/economics/sigma` — clean.
- `uv run mypy src/babylon/domain/economics/sigma` — clean (strict mode).
- No file outside this unit's declared write surface touched. No
  `engine/`, `simulation_engine.py`, `models/world_state.py`,
  `config/defines/`, `defines.yaml`, or `formulas/__init__.py` change.
- Three Director-ruling items recorded (D1/D2 folded, D3) for U4/U5 to carry
  forward; none resolved unilaterally by this unit.
