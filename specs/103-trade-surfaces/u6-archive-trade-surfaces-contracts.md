# P26 U6 — Archive Trade Surfaces: Seam Contracts (phase 1, backend projections)

**Pinned before code** (contract-pin-first). Successor to spec-103's web surfaces
(`BlocFlowLines` / `ImportExposurePanel` / `TradePanel`), whose frontend died with
`web/frontend` (spec-112). Charter: ADR160 U6 — "backend projections first
(client-agnostic `observe()` seam)". Coordinates with the raster-cutover lane:
**phase 1 changes no `tui/` or `rust/` file.**

## What exists to project (post-U2/U3, pre-U5)

- `GameSession._trade` (`TradeWiring`, U2): `external_nodes_phi`
  (`{node_id: Φ_year_usd}`), `county_exposure_by_external`, `weeks_per_year`,
  `start_year` — static per campaign.
- Per-tick DRAIN_EDGE flows: produced by `distribute_phi_week_to_counties` into the
  register, flushed into `PerTickTransactionEnvelope.boundary_register_rows`
  (U2). The register is EMPTY post-tick by contract — a live view needs the
  session to retain the last flush (see Contract 2).
- Post-U3: `bilateral_trade_tons` per node (FAF artifact, 2018–2024 coverage) and
  ERDI via the reference estate.
- NOT yet available (U5): blocs-as-alignments, flows over the substrate, σ acting
  on value flows. Phase-1 views must not fake them.

## Contract 1 — `projection/trade.py` (new module, Lane P shape)

- `TradeBlocView` (kind literal `"trade"`) joins the `ProjectionRecord`
  discriminated union in `projection/view_models.py` ("written to grow further" —
  its own docstring). Fields (all honest-`None`-able): `id` (node_id),
  `phi_year_inflow`, `phi_week_slice`, `bilateral_trade_value`,
  `bilateral_trade_tons`, `erdi_ratio`, `exposure_top` (top-N
  `(county_fips, weight)`), `last_tick_flow` (this bloc's most recent DRAIN_EDGE
  total), `tick`.
- `project_trade_bloc(node_id, *, trade, last_flows, tick)` — pure function over
  session-held data; degrades any missing input to honest `None`/empty (the
  `project_county` documented shape), never a crash.
- A `trade/overview` id projects the national fold: total Φ, per-bloc breakdown,
  flow-type summary (spec-103's `TradePanel` semantics).

## Contract 2 — session seams

- `advance_tick` retains its flushed rows: `self._last_boundary_rows = boundary_rows`
  (already computed for the envelope — zero extra work; empty when trade unwired).
- `subject_view` gains the `trade` kind: `trade/<node_id>` and `trade/overview`,
  routed from `self._trade` + `self._last_boundary_rows`. `trade` kind with
  `trade=None` (unwired campaign) returns `None` — honest absence, rendered by the
  existing "no longer resolvable" watchlist row.
- `known_subjects`/vault pages: NOT in phase 1 (baking trade pages into the vault
  drifts the golden estate → that lands with a declared §6.5 ceremony alongside
  U5's content, not as a phase-1 side effect).

## Contract 3 — client rendering (phase 2, EXPLICITLY out of scope here)

Rendering a `kind="trade"` record in the Textual shell and the Ratatui client is
the raster-lane coordination point (their per-kind templates + the Rust router's
BARE_KIND parity). Phase 1 ships the data seam only; a pinned `trade/canada`
subject resolves (backend-provable via `subject_view`) even before either client
grows a template.

## Gates

Scoped `test:q` on new tests; `mise run check` before commit; qa + vault gates
must stay byte-identical (no baker/page-set change in phase 1).
