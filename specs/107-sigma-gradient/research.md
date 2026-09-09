# Phase 0 Research — spec-107 σ-gradient

All findings verified in-repo against the tree at the head of
`101-trade-activation` (worktree `trade-activation`) on 2026-07-27, i.e. 9
days after program-10's own 2026-07-08 provenance date — several of its
claims have since drifted; each drift is cited against the current file.

## R1 — Dormant formula reuse audit: nothing in this package duplicates them

`src/babylon/formulas/unequal_exchange.py` (4 functions:
`calculate_exchange_ratio`, `calculate_unequal_exchange_rate`,
`calculate_value_transfer`, `prebisch_singer_effect`) and
`src/babylon/formulas/fundamental_theorem.py::calculate_labor_aristocracy_ratio`
are all registered in `engine/formula_registry.py` (lines 93, 112–115) with
**zero registry callers** (re-confirmed 2026-07-27 — unchanged from the P26
audit). These formulas operate on wages/labor-hours/exchange ratios, not on
σ's three production-structure ingredients (OCC, K/L, ℓ) — they are the
*consumers* σ will eventually feed (spec.md § Consumption seams: transfer
lever + consciousness coupling), not something this package's math
overlaps with. Confirmed by reading every line of both files: no function
signature or docstring in `sigma/` reproduces logic already present there.

## R2 — FAAt3.1ESI fixed-assets loader: still STAGED, unchanged since ratification

`rg -ni "FAAt3|fixed_assets|FixedAssets" src/babylon/reference/schema.py`
returns zero matches. Program-10 §4 marked this "STAGED — the one loader
this program needs. No ORM class yet"; that remains true 9 days (now,
generously, ~19 days project-time) later. No capital-stock-by-industry table
exists in the reference DB. `capital_stock.py`'s
`CapitalStockCalculator` (perpetual-inventory K from `TensorRegistry` flows,
Phase-5.2 wiring) is the closest hydrated substitute and is cited in
spec.md Decision D4 as the interim source for this package's `capital_stock`
inputs — not read by this package directly (D5: no I/O here).

## R3 — Hickel ERDI: confirmed national-aggregate-only, no per-bloc table

`reference/schema.py:1914-1965` (`FactHickelERDIAnnual`,
`__tablename__ = "fact_hickel_erdi_annual"`): columns `time_id`,
`scale_type` (values `Extensive`/`Intensive`/`Intensive_China_Inflection` —
a national time series selector, not a partner-country key), `erdi`,
`annual_drain_usd_billions`. The class's own docstring
(`reference/schema.py:1923-1925`) states: "Distinct from the retired
`fact_hickel_drain` resource-flow decomposition table (amputated 2026-07-17
per ADR075 ruling 1, A14)." Program-10 §3 named `FactHickelDrain` as a
second, separate ORM class supplying the anchor — no such class exists
under that name in the current schema; the retirement note suggests it may
have referred to `fact_hickel_drain`, which is now gone. Independently
confirmed by spec-101 R2 ("all external-node Φ is currently ZERO" because
Hickel is national-aggregate-only) — the same underlying data limitation
program-10's anchor design and spec-101's Φ-attribution both ran into,
from opposite directions.

## R4 — "Ricci Unequal Exchange": a genuine CSV, an unrelated same-named runtime table

Three distinct things share the word "Ricci" in this codebase, confirmed by
direct inspection:

1. **`src/babylon/data/reference/babylon_ricci_final.csv`** (51 data rows,
   read directly): columns `year, region_name, region_type, flow_direction,
   transfer_type, value_usd_billions, value_pct_gdp, signed_value,
   gvc_share_of_total, source_table, source_priority, region_granularity,
   edge_id`. `region_type ∈ {CORE, SEMI_PERIPHERY, PERIPHERY}`,
   `flow_direction ∈ {INFLOW, OUTFLOW}`, `transfer_type ∈ {GVC, TOTAL}`,
   `source_table` values like `Ricci_Table_6.2` (Andrea Ricci, *Unequal
   Exchange in the Age of Global Value Chains*, 2019 — a real academic
   global-value-chain unequal-exchange dataset, not a graph-curvature
   artifact). `region_name` values include `China`, `India`, `Russia and
   CSI`, `Southeast Asia`, `Sub-Saharan Africa`, `North America`, `Western
   Europe`, `EMU`, `OECD`, `Non-OECD`, `Advanced Economies`, `Emerging and
   Developing Economies`, `South Asia` — several map almost directly onto
   Program 26's 8 engine-node names, more cleanly than the trade-share
   crosswalk spec-101 D3 had to build for a differently-shaped bloc set.
2. **`fact_ricci_unequal_exchange`** — the sqlite table that (1) used to
   populate. **Amputated** 2026-07-17
   (`reports/amputation_demotion_20260717_011045.md`: `rows_before=29`;
   `ADR076_parquet_artifacts.yaml` R2: "the in-repo CSV is ALREADY
   canonical... drop/demote DB copy"). Not queryable from the reference DB
   today.
3. **`immutable_reference_ricci_unequal`** — a Postgres runtime table
   (`persistence/migrations/0010_immutable_reference_tables.sql:71`) that
   is, confusingly, populated by **Census bilateral trade**
   (`persistence/sqlite_hydrator.py::_copy_ricci_unequal`, reading
   `fact_trade_monthly` — `SUM(imports_usd_millions + exports_usd_millions)`
   grouped by country/year), not by (1) or (2) at all. Spec-101 R4
   independently found the same thing from the trade-value side:
   "`bilateral_trade_value` comes from `_copy_ricci_unequal`... NOT the
   program text's assumed Ricci UE series."

Net: program-10's assumed anchor input B ("FactRicciUnequalExchange, LOADED")
never existed as described; what's genuinely useful for the same purpose
(a real core/periphery/semi-periphery UE series) is (1), sitting unused
on disk. This is exactly the kind of drift spec.md Decision D3 escalates to
a Director ruling rather than silently re-wiring.

## R5 — Existing "domain math" package shapes surveyed for convention fit

Read in full before designing `sigma/`'s layout: `county_exposure.py`
(pure-ish, one sqlite-reading function + a documented no-silent-
renormalization discipline — directly cited in this package's
`ComponentWeights` validator), `capital_stock.py` (registry-backed, stateful
service class — not a fit for pure math), `reserve_army/types.py` +
`calculator.py` (the closest structural precedent: a `types.py` of frozen
Pydantic models + a separate calculator module of pure functions — this
package generalizes that split across five small modules instead of two,
since σ has five distinct pure-math stages rather than one).
`derived_rates.py` confirms the codebase's existing convention for
division-by-zero handling (`None`, not an exception) in one place
(`DerivedRateCalculator`) — this package instead raises `ValueError` on
degenerate inputs (zero wage bill, zero employment, zero-spread
distributions), matching `county_exposure.py`'s "hard failure, never silent"
convention instead, since σ math has no natural "None" sentinel value the
way `DerivedRates`' profit_rate/OCC do (their `None` specifically means "this
county has zero variable capital this tick," a real domain state; a
degenerate distribution or zero wage bill during calibration is closer to a
caller bug).

## R6 — GameDefines category-addition convention

`config/defines/_assembler.py`'s `GameDefines` composes ~20 named
sub-models (`crisis`, `economy`, `contradiction_field`, `reserve_army`, …),
one `.py` file per category under `config/defines/`. A `SpectrumDefines`
category (component weights, wage-gravitation rate) would follow this exact
pattern — confirmed but **not implemented**, since `config/defines/` and
`defines.yaml` regeneration are both outside this unit's write surface and
outside Program 26 §3's non-overlap covenant. Declared in tasks.md as a
pending addition for the wiring unit.

## R7 — pytest marker/task conventions for the red-phase tests

`pyproject.toml`'s `red_phase` marker ("TDD RED phase tests (intentionally
failing until GREEN phase)") is deselected by `mise run test:unit` (what
`mise run check` runs) but **not** by `mise run test:q` (a raw `pytest -q`
invocation, confirmed by reading `.mise.toml`'s `test:q` task body). Because
this unit's contract tests are designed to currently PASS (they assert
present-tense absence, not future-tense presence), this distinction doesn't
matter for this unit's own gate — but it means a future `mise run test:q`
against a broader path that includes genuinely-still-red TDD tests elsewhere
in the repo would not silently skip them the way `test:unit` does. No
existing `red_phase`-marked test currently ships anywhere in `tests/` (all
prior instances were flipped GREEN and had the marker dropped, per
`tests/README.md`'s "l markers retired 2026-07-08" note in several scenario
test files) — this unit's three tests are, as far as a repo-wide grep shows,
the first live `red_phase` tests in the current tree.

## R8 — Off-pipeline / non-overlap compliance check

Every file this unit reads (for research, not modification) —
`reference/schema.py`, `data-catalog.yaml`, `data-artifacts.yaml`,
`engine/formula_registry.py`, `engine/systems/contradiction_field.py`,
`domain/economics/{county_exposure,capital_stock,derived_rates}.py`,
`config/defines/_assembler.py`, `persistence/{sqlite_hydrator,
postgres_initialization}.py`, `persistence/migrations/
0010_immutable_reference_tables.sql` — is read-only in this unit. The three
directories actually written
(`specs/107-sigma-gradient/`, `src/babylon/domain/economics/sigma/`,
`tests/unit/domain/economics/sigma/`) do not intersect Program 26 §3's
forbidden list (`models/enums/events.py`, `formulas/__init__.py`,
`models/world_state.py`, `tests/baselines/*.json`,
`tools/regression_scenarios.py`, `web/game/engine_bridge.py`,
`engine/systems/*`, `domain/dialectics/instances/catalog.py`,
`config/defines/politics.py` + `defines.yaml` regeneration, `CLAUDE.md`,
`ai/wiring-doctrine.md`) nor this task's own additional forbidden list
(`engine/`, `simulation_engine.py`, `models/enums/events.py`,
`models/world_state.py`, `config/defines/`, `defines.yaml`, `CLAUDE.md`,
`ai/wiring-doctrine.md`, `ai/decisions/index.yaml`, `rust/`,
`src/babylon/tui/`, `web/`, `tests/baselines/`).
