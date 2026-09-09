# Feature Specification: Trade Surfaces in the Product UI

**Feature Branch**: `103-trade-surfaces`
**Created**: 2026-07-05
**Status**: In Progress
**Program**: 09 Full-Game Build — Lane W (web product). Stacks on `13438ac5` (095 HEAD).
**Deps**: spec-101 (real boundary flows) ✅, spec-093 (Territory Detail) ✅, spec-094 (INDEX) ✅.

## Overview

Spec-103 delivers the trade-UI surfaces across three product screens, per the
ratified trade-UI decision: **blocs are background noise; no interactive world
map; CONUS stays primary.** The surfaces are additive panels/sections on
existing screens — never a new world-map view.

Three surfaces:

1. **Wire INDEX** gains per-bloc price/flow lines — a `BlocFlowLines` section
   atop the story archive showing each external bloc's Φ inflow (DRAIN_EDGE)
   and unequal-exchange ratio (`erdi_ratio`) as sparkline time series.
2. **Territory Detail** gains an import-exposure provenance breakdown — an
   `ImportExposurePanel` showing which external blocs supply a county's import
   exposure, with a drill-down provenance chain (BabylonScriptValue over
   spec-100 weights + live `boundary_flow_register` flows) ending at
   reference-data citations.
3. **Analysis page** gains a trade panel — a `TradePanel` aggregating total Φ
   inflow, per-bloc breakdown, and flow-type summary across the whole session.

### Constitution III (AI observes, never controls)

All three endpoints are GET reads over already-persisted engine state:
- `boundary_flow_register` rows (per-tick dyadic flows written by the engine's
  economics stage — spec-062/101).
- `dynamic_external_node_state` rows (external bloc snapshots — spec-062).
- `county_exposure_by_external` reference weights (spec-100's table — read if
  present, degraded to empty if not yet built).

Zero `babylon.*` imports in frontend; zero engine state writes. The bridge
reads via the persistence pool's SQL (same pattern as
`_fetch_contradiction_field_rows`), never computes trade state itself.

### Degradation contract (Constitution III.8 — Aleksandrov Test)

The W-lane product DB (5432/babylon) does NOT carry `boundary_flow_register`
or `dynamic_external_node_state` — those live in the SIM DB (5433/babylon_test,
spec-096). Spec-100's `county_exposure_by_external` reference table is not yet
built. Therefore every bridge method degrades gracefully:

- Pool unavailable (SQLite dev/test) → `has_data: False`, honest zeros.
- `boundary_flow_register` empty → flow series empty, weights-only if present.
- `county_exposure_by_external` absent → weights empty, flows-only if present.
- Both empty → `has_data: False`.

The MSW fixtures provide FULL responses (with provenance chains + citations)
so frontend contract tests and the Playwright gate exercise the complete
drill-down chain. The real backend fills in as spec-100/096 lands.

## User Scenarios & Testing *(mandatory)*

### US1 — Wire INDEX shows per-bloc price/flow lines (Priority: P1, gate)

A player navigates to the Wire's INDEX tab. Above the story archive they see a
`BlocFlowLines` section: one row per external bloc (Canada, China, EU, India,
Sub-Saharan Africa, Latin America, Russia/CSI, Southeast Asia), each showing
the bloc label, a Φ-inflow sparkline (DRAIN_EDGE time series), a trade-value
sparkline, and the latest `erdi_ratio` (unequal-exchange index). Data comes
from real `boundary_flow_register` + `dynamic_external_node_state` rows —
never fixtures in production.

**Independent test**: pytest — `get_trade_flows` returns well-formed per-bloc
series from a mock persistence pool seeded with `boundary_flow_register` rows.
Vitest — `BlocFlowLines` renders bloc rows from a contract-faithful fixture;
asserts sparklines render; asserts `erdi_ratio` displays.

### US2 — Territory Detail shows import-exposure provenance chain (Priority: P1, gate)

A player drills into a county's Territory Detail. Below the Economy panel they
see an `ImportExposurePanel`: the county's total import-exposure value, a
breakdown by source bloc, and a drill-down provenance chain. Clicking a bloc
contributor expands to show: (a) the spec-100 exposure weight (with reference
citation: BEA I-O imports × QCEW county industry shares), and (b) the live
`boundary_flow_register` DRAIN_EDGE/TRADE_EDGE flow for that bloc→county pair
(with dynamic-table citation). The chain terminates at reference-data
citations.

**Independent test**: pytest — `get_county_import_exposure` returns a
breakdown with contributors, children, source refs, and citations from a mock
pool. Vitest — `ImportExposurePanel` renders the drill-down chain from a
contract-faithful fixture; asserts citations render; asserts the chain
terminates at reference-data sources.

### US3 — Analysis page shows trade panel (Priority: P1)

A player navigates to the Analysis page. A new `TradePanel` appears alongside
the existing time-series dashboard. It shows: total Φ inflow across all blocs
(session-cumulative), a per-bloc bar breakdown, and a flow-type summary
(DRAIN_EDGE / TRADE_EDGE / COMMUTE_OUT totals). Data comes from real
`boundary_flow_register` aggregates.

**Independent test**: pytest — `get_trade_panel` returns well-formed aggregate
data from a mock pool. Vitest — `TradePanel` renders the aggregate stats and
per-bloc bars from a contract-faithful fixture.

### US4 — Trade-flows endpoint contract (Priority: P1, gate)

`GET /api/games/{id}/trade-flows/` returns a `TradeFlowsPayload` with: the
latest tick, a `has_data` flag, and a `blocs` array. Each bloc entry has
`node_id`, `label`, `kind`, `latest` (phi_year_inflow, bilateral_trade_value,
bilateral_trade_tons, erdi_ratio), `phi_series`, and `trade_series`. Matches
`contracts/trade-flows.yaml`.

**Independent test**: Vitest — MSW contract test for
`/api/games/:id/trade-flows/`.

### US5 — County-exposure endpoint contract (Priority: P1, gate)

`GET /api/games/{id}/exposure/?county_fips=FIPS` returns an
`ExposurePayload` with: `county_fips`, `has_data`, `total_exposure`, a
`breakdown` (recursive contributors with source refs), and a `citations`
array (reference-data provenance). Matches `contracts/county-exposure.yaml`.

**Independent test**: Vitest — MSW contract test for
`/api/games/:id/exposure/?county_fips=`.

### US6 — Trade-panel endpoint contract (Priority: P1)

`GET /api/games/{id}/trade-panel/` returns a `TradePanelPayload` with: the
latest tick, `has_data`, `total_phi_inflow`, `total_trade`, a `blocs`
breakdown, and a `flow_types` summary. Matches `contracts/trade-panel.yaml`.

**Independent test**: Vitest — MSW contract test for
`/api/games/:id/trade-panel/`.

## Requirements *(mandatory)*

- **FR-103-01**: `EngineBridge.get_trade_flows(session_id)` MUST read
  `boundary_flow_register` rows (per-tick, grouped by source_node_id +
  flow_type) and `dynamic_external_node_state` rows (latest tick per node)
  via the persistence pool's SQL (same pattern as
  `_fetch_contradiction_field_rows`). Returns a `TradeFlowsPayload` dict.
  Degrades to `has_data: False` with an empty `blocs` list when the pool is
  unavailable or both tables are empty/absent.

- **FR-103-02**: `EngineBridge.get_county_import_exposure(session_id,
  county_fips)` MUST return an import-exposure provenance breakdown — a
  BabylonScriptValue-style `{value, breakdown}` where the breakdown's
  contributors are per-bloc, each with children tracing to (a) the spec-100
  `county_exposure_by_external` weight (source: `reference_table`) and (b)
  the live `boundary_flow_register` DRAIN_EDGE/TRADE_EDGE flow for that
  bloc→county pair (source: `dynamic_table`). The `citations` array carries
  the terminal reference-data provenance (BEA I-O, QCEW, Hickel drain). Degrades
  to `has_data: False` with honest zeros when no data is available.

- **FR-103-03**: `EngineBridge.get_trade_panel(session_id)` MUST return an
  aggregate trade panel — session-cumulative Φ inflow, per-bloc breakdown, and
  flow-type summary — from `boundary_flow_register` aggregates + external node
  state. Degrades to `has_data: False` with honest zeros.

- **FR-103-04**: `GET /api/games/{id}/trade-flows/` MUST serve the
  `TradeFlowsPayload` via a new `game_trade_flows` Django view, following the
  `game_wire` / `game_economy` pattern.

- **FR-103-05**: `GET /api/games/{id}/exposure/` MUST serve the
  `ExposurePayload` via a new `game_county_exposure` Django view, accepting a
  `?county_fips=` query parameter (required).

- **FR-103-06**: `GET /api/games/{id}/trade-panel/` MUST serve the
  `TradePanelPayload` via a new `game_trade_panel` Django view.

- **FR-103-07**: The frontend `useTradeFlows(gameId)` hook MUST poll
  `GET /api/games/{id}/trade-flows/` on a 2s interval, exposing
  `{data, loading, error, refresh}`.

- **FR-103-08**: The frontend `useCountyExposure(gameId, countyFips)` hook
  MUST poll `GET /api/games/{id}/exposure/?county_fips=` on a 2s interval.

- **FR-103-09**: The frontend `useTradePanel(gameId)` hook MUST poll
  `GET /api/games/{id}/trade-panel/` on a 2s interval.

- **FR-103-10**: The `BlocFlowLines` component MUST render per-bloc rows in
  the Wire INDEX tab, each with a Φ-inflow sparkline, trade-value sparkline,
  and `erdi_ratio`. Uses Cold Collapse tokens (spec-090).

- **FR-103-11**: The `ImportExposurePanel` component MUST render in Territory
  Detail a drill-down provenance chain: total exposure → per-bloc contributors
  → spec-100 weight + live flow → reference-data citations. Uses Cold Collapse
  tokens. The drill-down is click-to-expand (Radix Popover or disclosure),
  reusing the `BreakdownTree` visual pattern.

- **FR-103-12**: The `TradePanel` component MUST render in the Analysis page
  the aggregate trade stats, per-bloc bars, and flow-type summary. Uses Cold
  Collapse tokens.

- **FR-103-13**: MSW handlers MUST exist for `/api/games/:id/trade-flows/`,
  `/api/games/:id/exposure/`, `/api/games/:id/trade-panel/` in `handlers.ts`.
  The exposure handler MUST vary by `county_fips` query param.

- **FR-103-14**: Three new TypeScript types MUST be added: `TradeFlowsPayload`,
  `ExposurePayload`, `TradePanelPayload` in `types/trade.ts`.

## Success Criteria *(mandatory)*

- **SC-103-01**: `mise run web:check` green (tsc + eslint + prettier + Vitest).
  The 3 pre-existing tick-resolution-page failures are acceptable (documented
  pre-existing).
- **SC-103-02**: `PYTHONPATH=src uv run pytest tests/unit/web/ -q` green,
  including the 3 new bridge method contract tests and 3 new API view tests.
- **SC-103-03**: Trade-flows contract test passes (bridge returns real
  per-bloc series, not fixtures).
- **SC-103-04**: County-exposure contract test passes — the breakdown renders
  with a drill-down provenance chain ending at reference-data citations.
- **SC-103-05**: MSW contract tests for all 3 new endpoints pass.
- **SC-103-06**: Playwright: a county's import exposure renders with a
  drill-down provenance chain ending at reference-data citations (owner-run,
  gated on `SPEC061_TEST_SESSION_ID`).

## Known Gaps (documented, not fixed here)

1. **Spec-100 reference table absent**: `county_exposure_by_external` is not
   yet built (spec-100 is Lane D, unbuilt). The exposure weights degrade to
   empty; the provenance chain still renders the live `boundary_flow_register`
   flows when present. When spec-100 lands, the weights populate without a
   frontend change (the contract is forward-compatible).

2. **SIM DB split (spec-096)**: `boundary_flow_register` and
   `dynamic_external_node_state` live in the SIM DB (5433/babylon_test), not
   the product DB (5432/babylon). The bridge's persistence pool reads from
   whichever DB the runtime configures. In dev/test (SQLite), the methods
   degrade to `has_data: False`. When spec-096's two-DB alias map lands, the
   bridge reads the SIM DB for these tables.

3. **No interactive world map**: Per the ratified decision, blocs are
   background noise. There is no world-map view — the blocs surface only as
   sparklines/bars within CONUS-primary screens. A future spec may add a bloc
   detail screen if needed.

4. **COMMUTE_OUT flows**: spec-101 gates COMMUTE_OUT emission until 098-LODES.
  The trade panel surfaces a `COMMUTE_OUT` flow-type total that will read as
  zero until 098 lands. The panel renders the zero honestly.
